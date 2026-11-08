//! Sound playback: vanilla's `SoundEngine` bookkeeping (`sound_engine`) over a
//! small mixer standing in for OpenAL: one channel per source with its gain,
//! pitch, linear distance attenuation (`AL_LINEAR_DISTANCE`, reference 0,
//! rolloff 1), looping, relative flag and source state, Ogg/Vorbis decoding
//! (streams on a worker thread) and OpenAL Soft's stereo panning. Stereo files play as they are,
//! as OpenAL plays a stereo buffer, and Directional Audio renders mono sources for headphones
//! (`ear_settings`).
use crate::Result;
use crate::sound_engine::{
    Attenuation, Backend, Bound, ChannelId, InstanceId, PlayResult, SoundEngine, SoundInstance,
    Sounds, TickContext,
};
use crate::sound_events::{Sound, SoundRegistry};
#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use horizon_core::random::{LegacyRandom, RandomSource};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// An OpenAL source's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Initial,
    Playing,
    Paused,
    Stopped,
}
/// An OpenAL source.
struct Channel {
    /// Samples (a stereo file's interleaved), their rate and channel count (1 or 2).
    samples: Option<(Arc<Vec<f32>>, u32, u16)>,
    position: f64,
    state: State,
    gain: f32,
    pitch: f32,
    /// `AL_LINEAR_DISTANCE` max distance, or no attenuation.
    linear: Option<f32>,
    looping: bool,
    at: [f32; 3],
    relative: bool,
    /// Whose sounds these are: 0 the game's, another splitscreen player's (`Audio::share`).
    owner: u8,
    /// Its world (`Mixer::world_id`; 0 none).
    world: u32,
    /// The sound file it started and when, to tell another player's copy of it.
    started: Option<(String, std::time::Instant)>,
    /// Directional Audio's ears for it.
    ears: Ears,
    /// Sound physics' last word on it (`Audio::set_physics`), and when.
    physics: Option<(Physics, std::time::Instant)>,
    /// The muffling filter's last output.
    muffled: f32,
}

/// What the blocks around a world sound do to it (sound physics, Sound Physics Remastered's
/// port): measured by the game from the world (`game/android_bundled/sound_physics.rs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Physics {
    /// The gain of the sound heard directly, through what's between.
    pub gain: f32,
    /// How much of its highs get through (the gain at 5 kHz, as OpenAL's EFX low-pass).
    pub highs: f32,
    /// How much feeds each echo of the space around it, shortest first
    /// (`REVERB_DECAYS`).
    pub sends: [f32; 4],
}
#[derive(Default)]
struct Mixer {
    channels: HashMap<ChannelId, Channel>,
    listener: [f64; 3],
    listener_yaw: f32,
    listener_pitch: f32,
    /// Splitscreen: the other players' cameras (position, yaw, pitch) by owner. A world sound is
    /// heard from the nearest camera, and panned half as wide while several share the speakers.
    listeners: HashMap<u8, ([f64; 3], f32, f32)>,
    /// Each camera's world by owner, as ids: the callback compares and copies integers.
    worlds: HashMap<u8, u32>,
    /// World names by id - 1 (`world_id`).
    world_names: Vec<String>,
    /// The callback's cameras (position, basis, world), kept to be refilled without allocating.
    cameras: Vec<([f64; 3], Basis, u32)>,
    limiter_peak: f32,
    /// `options.directionalAudio`: mono sources rendered for headphones.
    directional: bool,
    /// Sound physics on (`Audio::set_sound_physics`), and the listener under water.
    physics: bool,
    underwater: bool,
    /// The echoes of sound physics, made when first needed, and their input this buffer.
    reverb: Option<Box<Reverb>>,
    /// The output device the stream was opened on (`Library.currentDeviceName`).
    #[cfg(feature = "audio")]
    device_name: String,
    /// The stream reported its device gone (`Library.isCurrentDeviceDisconnected`).
    #[cfg(feature = "audio")]
    device_lost: bool,
}
#[cfg(feature = "audio")]
type DeviceProbe = (Option<String>, Option<String>, Vec<String>);

/// A camera's forward, up and right vectors.
type Basis = ([f64; 3], [f64; 3], [f64; 3]);
impl Mixer {
    /// The id of world `name` (1 and up), given once per name.
    fn world_id(&mut self, name: &str) -> u32 {
        let i = match self.world_names.iter().position(|w| w == name) {
            Some(i) => i,
            None => {
                self.world_names.push(name.to_owned());
                self.world_names.len() - 1
            }
        };
        i as u32 + 1
    }
}
/// A sound the player could have heard (`SoundEventListener.onPlaySound`):
/// what the subtitle overlay needs.
#[derive(Clone, Debug, PartialEq)]
pub struct Heard {
    /// The `subtitle` translation key of the sound event.
    pub subtitle: String,
    /// The instance's position (relative sounds report their raw coordinates).
    pub at: Option<[f64; 3]>,
    /// `SoundEngine.getRange`: infinite for relative and non-attenuated sounds.
    pub range: f32,
    pub looping: bool,
}
/// The mixer side of the engine (`Backend`).
pub struct MixerBackend {
    mixer: Arc<Mutex<Mixer>>,
    root: PathBuf,
    cache: Arc<Mutex<SoundCache>>,
    /// The decoder thread for static sounds, started with the first one.
    decoder: Option<std::sync::mpsc::Sender<Decode>>,
    next: ChannelId,
    live: bool,
    /// Whose channels these are (`Channel::owner`).
    owner: u8,
    world: u32,
    /// How many other players share this output: alone, a sound never looks for its copy.
    shares: Arc<std::sync::atomic::AtomicUsize>,
}
/// A splitscreen player's backend leaving: its camera and sounds go with it.
impl Drop for MixerBackend {
    fn drop(&mut self) {
        if self.owner == 0 {
            return;
        }
        self.shares
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut m) = self.mixer.lock() {
            m.listeners.remove(&self.owner);
            m.worlds.remove(&self.owner);
            let owner = self.owner;
            m.channels.retain(|_, c| c.owner != owner);
        }
    }
}
/// Splitscreen: each shared backend's channel ids start at a block of their own, never handed out
/// again, so a player in a seat used before never reuses an id a departed one's decoder may
/// still attach to. The game's own backend counts from 0, as before.
static SHARED_CHANNELS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Two players' copies of one sound: started within this of each other.
const SAME_SOUND: std::time::Duration = std::time::Duration::from_millis(150);
/// Decoded samples, their rate and channel count (`None`: the file did not load).
type Samples = Option<(Arc<Vec<f32>>, u32, u16)>;
/// Decoded static sounds (`SoundBufferLibrary`'s buffers), dropped least
/// recently used first past `CACHE_BYTES`: kept forever, every sound heard in
/// a session stayed (up to 1.4 GiB of samples, villages playing hundreds).
#[derive(Default)]
struct SoundCache {
    entries: HashMap<String, (Samples, u64)>,
    bytes: usize,
    clock: u64,
}
#[cfg(any(test, not(target_os = "android")))]
const CACHE_BYTES: usize = 64 << 20;
impl SoundCache {
    fn get(&mut self, key: &str) -> Option<Samples> {
        self.clock += 1;
        let entry = self.entries.get_mut(key)?;
        entry.1 = self.clock;
        Some(entry.0.clone())
    }
    fn insert(&mut self, key: String, samples: Samples) {
        self.clock += 1;
        self.bytes += Self::size(&samples);
        if let Some((old, _)) = self.entries.insert(key, (samples, self.clock)) {
            self.bytes -= Self::size(&old);
        }
        #[cfg(target_os = "android")]
        let budget = crate::android_runtime::sound_cache_budget();
        #[cfg(not(target_os = "android"))]
        let budget = CACHE_BYTES;
        // Playing channels own their Arc; eviction never interrupts a sound.
        let strict =
            cfg!(target_os = "android") && crate::android_runtime::cache_budget_overridden();
        while self.bytes > budget
            && (self.entries.len() > 1 || (strict && !self.entries.is_empty()))
        {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, (_, used))| *used)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            if let Some((gone, _)) = self.entries.remove(&oldest) {
                self.bytes -= Self::size(&gone);
            }
        }
    }
    fn size(samples: &Samples) -> usize {
        samples
            .as_ref()
            .map_or(0, |(s, _, _)| s.len() * std::mem::size_of::<f32>())
    }
}
/// A static sound to decode for a channel waiting on it.
struct Decode {
    key: String,
    file: PathBuf,
    channel: ChannelId,
}
/// Give a channel waiting on its sound the decoded samples and start it, as
/// the buffer future attaching and playing the source. A source stopped or
/// released meanwhile stays so.
fn attach(mixer: &Mutex<Mixer>, channel: ChannelId, samples: Samples, looping: bool) {
    if let Some(samples) = samples
        && let Ok(mut m) = mixer.lock()
        && let Some(c) = m.channels.get_mut(&channel)
        && c.state == State::Initial
    {
        c.samples = Some(samples);
        c.position = 0.0;
        c.state = State::Playing;
        c.looping |= looping;
    }
}
/// The open output stream. A build without the `audio` feature (the server build, which must not
/// need libasound) never has one.
#[cfg(feature = "audio")]
type Stream = cpal::Stream;
#[cfg(not(feature = "audio"))]
type Stream = std::convert::Infallible;
pub struct Audio {
    _stream: Option<Stream>,
    device_name: String,
    pub engine: SoundEngine,
    backend: MixerBackend,
    /// Record `Heard` sounds (the subtitle overlay is listening).
    pub record_heard: bool,
    pub listener: [f64; 3],
    pub listener_yaw: f32,
    pub master: f32,
    /// Seeds of positional plays (the client level random's `nextLong`).
    random: LegacyRandom,
    /// A `SoundSource` index the next sound plays on (`/playsound`).
    source_override: Option<usize>,
    /// Positional sounds played since the last drain, kept while the world
    /// is open to the network so guests hear them too.
    pub capture: Option<Vec<(String, [f64; 3], f32, f32)>>,
    /// How many times each positional sound played (the control socket's `status`),
    /// with or without an audio device.
    pub heard: std::collections::BTreeMap<String, u32>,
    /// How many times each selection was stopped ("record *", "* minecraft:…", `*` for
    /// any), with or without an audio device, as `heard`.
    pub stopped: std::collections::BTreeMap<String, u32>,
    /// The volume sliders' `SoundPreviewHandler`.
    preview: crate::sound_preview::SoundPreview,
    /// `SoundEngine.lastSeenDevices`: the system default output last seen.
    #[cfg(feature = "audio")]
    default_seen: Option<String>,
    /// The ALSA cards (Linux) or Core Audio devices (macOS) last seen: listing
    /// the output devices probes every one, so it waits for these to change
    /// (`card_fingerprint`).
    #[cfg(feature = "audio")]
    cards_seen: Option<String>,
    /// The other half of `lastSeenDevices`: every output device last seen.
    #[cfg(feature = "audio")]
    devices_seen: Vec<String>,
    /// Wall-clock time of the last device query; the launcher ticks per frame.
    #[cfg(feature = "audio")]
    device_probe_started: std::time::Instant,
    /// One in-flight device query. CPAL discovery can block for hundreds of
    /// milliseconds on Android, so the render thread only receives its result.
    #[cfg(feature = "audio")]
    device_probe: Option<std::sync::mpsc::Receiver<DeviceProbe>>,
}
/// A sound file's samples, rate and channel count. A stereo file keeps its two channels
/// interleaved (OpenAL plays a stereo buffer as it is); any other layout is mixed to mono.
fn decode(path: &Path) -> Option<(Vec<f32>, u32, u16)> {
    use symphonia::core::{
        audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
        meta::MetadataOptions, probe::Hint,
    };
    let file = fs::File::open(path).ok()?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("ogg");
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let mut format = probed.format;
    let track = format.default_track()?;
    let id = track.id;
    let rate = track.codec_params.sample_rate?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .ok()?;
    let mut out = Vec::new();
    // The first packet's layout: 2 stereo, 1 anything else.
    let mut width = 0u16;
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != id {
            continue;
        }
        let Ok(buffer) = decoder.decode(&packet) else {
            continue;
        };
        let spec = *buffer.spec();
        let channels = spec.channels.count().max(1);
        let mut samples = SampleBuffer::<f32>::new(buffer.capacity() as u64, spec);
        samples.copy_interleaved_ref(buffer);
        if width == 0 {
            width = if channels == 2 { 2 } else { 1 };
        }
        for frame in samples.samples().chunks(channels) {
            if width == 2 && frame.len() == 2 {
                out.extend_from_slice(frame);
            } else {
                let mono = frame.iter().sum::<f32>() / channels as f32;
                out.extend(std::iter::repeat_n(mono, usize::from(width)));
            }
        }
    }
    Some((out, rate, width.max(1)))
}
/// A sound file without its variant number (`mob/creeper/say2.ogg` is `mob/creeper/say`): each
/// player's game picks its own variant of the same event.
// ponytail: variants named by a trailing number, as vanilla's are; the event name if a pack breaks it.
fn variantless(key: &str) -> &str {
    key.trim_end_matches(".ogg")
        .trim_end_matches(|c: char| c.is_ascii_digit())
}
impl MixerBackend {
    /// Splitscreen: another player's game started this same sound here just now (both clients
    /// heard it); this copy stays stopped, the first is heard from the nearest camera.
    ///
    /// A copy is recognised only by its file (without the variant), world, place and time: the
    /// games pick variants on their own and play many sounds client-side (steps, digging), so
    /// nothing marks two copies of one event. Two different events alike in all four (two
    /// players' steps on grass, side by side, within 150 ms) are heard once. That is accepted:
    /// at that distance and timing they sound like one; a doubled sound is the louder fault.
    fn duplicate(&self, channel: ChannelId, (namespace, path): (&str, &str)) -> bool {
        // Alone: nobody else's copy, and no lock taken for it.
        if self.shares.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            return false;
        }
        let Ok(mut m) = self.mixer.lock() else {
            return false;
        };
        // Alone (no other player's camera), or a sound by the listener (a click, music): its own.
        let Some(c) = m.channels.get(&channel).filter(|c| !c.relative) else {
            return false;
        };
        if m.listeners.is_empty() {
            return false;
        }
        let key = format!("{namespace}:{path}");
        let key = variantless(&key);
        let at = c.at;
        let world = c.world;
        let twin = m.channels.values().any(|o| {
            o.owner != self.owner
                && o.world == world
                && !o.relative
                && o.started
                    .as_ref()
                    .is_some_and(|(k, t)| k == key && t.elapsed() < SAME_SOUND)
                && (0..3).map(|i| (o.at[i] - at[i]).powi(2)).sum::<f32>() < 4.0
        });
        if let Some(c) = m.channels.get_mut(&channel) {
            if twin {
                c.state = State::Stopped;
            } else {
                c.started = Some((key.to_owned(), std::time::Instant::now()));
            }
        }
        twin
    }
    fn with(&self, channel: ChannelId, f: impl FnOnce(&mut Channel)) {
        if let Ok(mut m) = self.mixer.lock()
            && let Some(c) = m.channels.get_mut(&channel)
        {
            f(c);
        }
    }
}
impl Backend for MixerBackend {
    fn create(&mut self, _streaming: bool) -> Option<ChannelId> {
        self.next += 1;
        let id = self.next;
        self.mixer.lock().ok()?.channels.insert(
            id,
            Channel {
                samples: None,
                position: 0.0,
                state: State::Initial,
                gain: 1.0,
                pitch: 1.0,
                linear: None,
                looping: false,
                at: [0.0; 3],
                relative: false,
                owner: self.owner,
                world: self.world,
                started: None,
                ears: Ears::default(),
                physics: None,
                muffled: 0.0,
            },
        );
        Some(id)
    }
    fn destroy(&mut self, channel: ChannelId) {
        if let Ok(mut m) = self.mixer.lock() {
            m.channels.remove(&channel);
        }
    }
    fn set_pitch(&mut self, channel: ChannelId, pitch: f32) {
        self.with(channel, |c| c.pitch = pitch);
    }
    fn set_volume(&mut self, channel: ChannelId, volume: f32) {
        self.with(channel, |c| c.gain = volume);
    }
    fn set_attenuation(&mut self, channel: ChannelId, linear: Option<f32>) {
        self.with(channel, |c| c.linear = linear);
    }
    fn set_looping(&mut self, channel: ChannelId, looping: bool) {
        self.with(channel, |c| c.looping = looping);
    }
    fn set_position(&mut self, channel: ChannelId, p: [f64; 3]) {
        self.with(channel, |c| c.at = [p[0] as f32, p[1] as f32, p[2] as f32]);
    }
    fn set_relative(&mut self, channel: ChannelId, relative: bool) {
        self.with(channel, |c| c.relative = relative);
    }
    /// `getCompleteBuffer` (cached, else decoded on the decoder thread) or
    /// `getStream` (decoded on a thread of its own), then attach and
    /// `alSourcePlay`. A sound that cannot load leaves its source initial,
    /// as a failed buffer future does in vanilla.
    fn start(&mut self, channel: ChannelId, sound: &Sound, looping: bool) {
        if !self.live {
            return;
        }
        let (namespace, path) = sound.path();
        let file = self.root.join("assets").join(namespace).join(&path);
        // A looping sound follows its entity in one client's tracking only: each keeps its own.
        if !looping && self.duplicate(channel, (namespace, &path)) {
            return;
        }
        if sound.stream {
            // A streamed sound loops by restarting its stream.
            let mixer = self.mixer.clone();
            std::thread::spawn(move || {
                let samples = decode(&file).map(|(s, r, w)| (Arc::new(s), r, w));
                attach(&mixer, channel, samples, looping);
            });
            return;
        }
        let key = format!("{namespace}:{path}");
        let cached = self.cache.lock().ok().and_then(|mut c| c.get(&key));
        if let Some(samples) = cached {
            attach(&self.mixer, channel, samples, false);
            return;
        }
        let decoder = self.decoder.get_or_insert_with(|| {
            let (send, receive) = std::sync::mpsc::channel::<Decode>();
            let (mixer, cache) = (self.mixer.clone(), self.cache.clone());
            std::thread::Builder::new()
                .name("sound decoder".into())
                .spawn(move || {
                    for job in receive {
                        // Queued twice before the first decode finished.
                        let cached = cache.lock().ok().and_then(|mut c| c.get(&job.key));
                        let samples = cached.unwrap_or_else(|| {
                            let samples = decode(&job.file).map(|(s, r, w)| (Arc::new(s), r, w));
                            if let Ok(mut c) = cache.lock() {
                                c.insert(job.key, samples.clone());
                            }
                            samples
                        });
                        attach(&mixer, job.channel, samples, false);
                    }
                })
                .expect("sound decoder thread");
            send
        });
        let _ = decoder.send(Decode { key, file, channel });
    }
    /// `alSourceStop`: any state becomes stopped, an initial source too (a
    /// sound still decoding then never starts).
    fn stop(&mut self, channel: ChannelId) {
        self.with(channel, |c| c.state = State::Stopped);
    }
    fn pause(&mut self, channel: ChannelId) {
        self.with(channel, |c| {
            if c.state == State::Playing {
                c.state = State::Paused;
            }
        });
    }
    fn unpause(&mut self, channel: ChannelId) {
        self.with(channel, |c| {
            if c.state == State::Paused {
                c.state = State::Playing;
            }
        });
    }
    fn stopped(&mut self, channel: ChannelId) -> bool {
        self.mixer
            .lock()
            .map(|m| {
                m.channels
                    .get(&channel)
                    .is_none_or(|c| c.state == State::Stopped)
            })
            .unwrap_or(true)
    }
}
/// OpenAL Soft's stereo gains `[left, right]` for a mono source in a
/// direction given in listener space (right, up, forward; any length).
/// Vanilla's context (`Library.createAttributes`: HRTF off) on a stereo
/// output renders pairwise: front azimuths are stretched by 1.5
/// (`ScaleAzimuthFront`, so 60 degrees off-centre pans fully) and the
/// direction is decoded by the first-order stereo decoder, 0.5 W, 0.5 Y and
/// 0.0957 X in this scaling. A centred source gets 0.596 on each side, one
/// behind 0.404, one above or below 0.5, one at the source's own position
/// is panned to the front. Matches OpenAL Soft 1.25.2 (LWJGL 3.4.3, the
/// pinned client's) rendered through `ALC_SOFT_loopback` within 1e-5.
fn stereo_gains(right: f32, up: f32, forward: f32) -> [f32; 2] {
    let length = (right * right + up * up + forward * forward).sqrt();
    if length <= f32::EPSILON {
        return stereo_gains(0.0, 0.0, 1.0);
    }
    let elevation = (up / length).clamp(-1.0, 1.0).asin();
    let mut azimuth = right.atan2(forward);
    if azimuth.abs() < std::f32::consts::FRAC_PI_2 {
        azimuth = (azimuth.abs() * 1.5)
            .min(std::f32::consts::FRAC_PI_2)
            .copysign(azimuth);
    }
    let side = azimuth.sin() * elevation.cos();
    let front = azimuth.cos() * elevation.cos() * 0.095_660_8;
    [0.5 - 0.5 * side + front, 0.5 + 0.5 * side + front]
}
/// The gain OpenAL applies for distance (`AL_LINEAR_DISTANCE` with reference
/// distance 0 and rolloff 1: the distance clamps to the max distance).
fn distance_gain(linear: Option<f32>, distance: f32) -> f32 {
    match linear {
        None => 1.0,
        Some(max) if max > 0.0 => (1.0 - distance.min(max) / max).clamp(0.0, 1.0),
        Some(_) => 0.0,
    }
}
/// Directional Audio's head (`options.directionalAudio`, OpenAL Soft's HRTF in
/// vanilla): a sphere with an ear at each side, after Brown and Duda's structural
/// model ("A Structural Model for Binaural Sound Synthesis", 1998). The ear away
/// from a source hears it later (up to 0.66 ms) and duller (the head's shadow),
/// the near one brighter, and a source behind loses its highs to the outer ears.
/// It cues what a measured HRTF set cues, without being OpenAL Soft's set.
const HEAD_RADIUS: f32 = 0.0875;
const SPEED_OF_SOUND: f32 = 343.0;
/// The ears' delay line in samples: the longest delay is 63 samples at 96 kHz.
const EAR_RING: usize = 128;
/// The binaural mix's level: a source ahead is about as loud as panned
/// (`stereo_gains`: 0.596 a side).
const BINAURAL_LEVEL: f32 = 0.6;

/// A mono source's ears for Directional Audio: its last samples and each ear's
/// delay and filters, kept from one buffer to the next.
#[derive(Clone, Copy)]
struct Ears {
    ring: [f32; EAR_RING],
    /// Where the newest sample goes.
    at: usize,
    /// Each ear's delay at the end of the last buffer (`[left, right]`), once started.
    delay: Option<[f32; 2]>,
    /// Each ear's head-shadow filter: its last input and output.
    shadow: [[f32; 2]; 2],
    /// Each ear's rear low-pass output.
    back: [f32; 2],
}
impl Default for Ears {
    fn default() -> Self {
        Self {
            ring: [0.0; EAR_RING],
            at: 0,
            delay: None,
            shadow: [[0.0; 2]; 2],
            back: [0.0; 2],
        }
    }
}
/// One buffer's ears for a source in direction `dir` (listener space: right, up,
/// forward; any length).
#[derive(Clone, Copy, Debug, PartialEq)]
struct EarSettings {
    /// Each ear's delay in samples (`[left, right]`).
    delay: [f32; 2],
    /// Each ear's head-shadow filter `[b0, b1, a1]`: one pole and one zero, unity
    /// at low frequencies and `alpha` at high ones.
    shadow: [[f32; 3]; 2],
    /// The rear low-pass's coefficient: 1 passes everything (a source in front).
    back: f32,
}
fn ear_settings(dir: [f32; 3], rate: u32) -> EarSettings {
    use std::f32::consts::{FRAC_PI_2, PI};
    let length = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let [right, _, forward] = if length <= f32::EPSILON {
        [0.0, 0.0, 1.0]
    } else {
        dir.map(|d| d / length)
    };
    let fs = rate.max(1) as f32;
    let k = 2.0 * fs;
    let beta = 2.0 * SPEED_OF_SOUND / HEAD_RADIUS;
    let head = HEAD_RADIUS / SPEED_OF_SOUND;
    // `side`: the cosine of the angle between the source and this ear's side.
    let ear = |side: f32| {
        let theta = side.clamp(-1.0, 1.0).acos();
        let delay = if theta < FRAC_PI_2 {
            head * (1.0 - theta.cos())
        } else {
            head * (1.0 + theta - FRAC_PI_2)
        };
        // 6 dB brighter facing the source, 20 dB duller at 150 degrees.
        let alpha = 1.05 + 0.95 * (theta / 150f32.to_radians() * PI).cos();
        // The bilinear transform of (alpha s + beta) / (s + beta).
        let shadow = [
            (alpha * k + beta) / (k + beta),
            (beta - alpha * k) / (k + beta),
            (beta - k) / (k + beta),
        ];
        (delay * fs, shadow)
    };
    let (left_delay, left) = ear(-right);
    let (right_delay, right_shadow) = ear(right);
    // Behind: a low-pass from no filter (ahead or level) to 5 kHz (straight behind).
    let behind = (-forward).max(0.0);
    let full = 1.0 - (-2.0 * PI * 5_000.0 / fs).exp();
    EarSettings {
        delay: [left_delay, right_delay],
        shadow: [left, right_shadow],
        back: 1.0 - behind * (1.0 - full),
    }
}
/// One sample `s` of a mono source through its ears, each delayed `delay` samples.
fn hear(ears: &mut Ears, s: f32, delay: [f32; 2], set: &EarSettings) -> [f32; 2] {
    ears.ring[ears.at] = s;
    let mut out = [0.0; 2];
    for e in 0..2 {
        let d = delay[e].clamp(0.0, (EAR_RING - 2) as f32);
        let whole = d.floor();
        let t = d - whole;
        let newer = (ears.at + EAR_RING - whole as usize) % EAR_RING;
        let older = (newer + EAR_RING - 1) % EAR_RING;
        let x = ears.ring[newer] * (1.0 - t) + ears.ring[older] * t;
        let [b0, b1, a1] = set.shadow[e];
        let [x1, y1] = ears.shadow[e];
        let mut y = b0 * x + b1 * x1 - a1 * y1;
        if y.abs() < 1e-20 {
            y = 0.0;
        }
        ears.shadow[e] = [x, y];
        ears.back[e] += set.back * (y - ears.back[e]);
        if ears.back[e].abs() < 1e-20 {
            ears.back[e] = 0.0;
        }
        out[e] = ears.back[e];
    }
    ears.at = (ears.at + 1) % EAR_RING;
    out
}
/// The echoes' decay times in seconds, short room to great cave: a sound's `Physics::sends`
/// feed each.
pub const REVERB_DECAYS: [f32; 4] = [0.15, 0.55, 1.68, 4.14];
/// Each echo's four delay lines, in milliseconds (no two share a factor).
const REVERB_LINES: [[f32; 4]; 4] = [
    [7.3, 9.1, 11.3, 13.7],
    [23.1, 29.3, 31.7, 37.9],
    [41.1, 47.3, 53.3, 61.7],
    [67.1, 73.3, 79.7, 89.3],
];
/// How loud the echoes come back.
const REVERB_LEVEL: f32 = 0.35;

/// One echo: a four-line feedback delay network, its lines mixed by a Householder reflection
/// (lossless) and each line's gain set for the decay time, highs damped on every pass.
struct Echo {
    lines: [Vec<f32>; 4],
    at: [usize; 4],
    gain: [f32; 4],
    damped: [f32; 4],
    damping: f32,
}
impl Echo {
    fn new(decay: f32, lines_ms: [f32; 4], rate: u32) -> Self {
        let fs = rate.max(1) as f32;
        let lengths = lines_ms.map(|ms| ((ms / 1000.0 * fs) as usize).max(1));
        Self {
            lines: lengths.map(|n| vec![0.0; n]),
            at: [0; 4],
            gain: lengths.map(|n| 10f32.powf(-3.0 * n as f32 / fs / decay)),
            damped: [0.0; 4],
            damping: 1.0 - (-2.0 * std::f32::consts::PI * 6_000.0 / fs).exp(),
        }
    }
    fn step(&mut self, input: f32) -> [f32; 2] {
        let mut out = [0.0f32; 4];
        for i in 0..4 {
            let read = self.lines[i][self.at[i]];
            self.damped[i] += self.damping * (read - self.damped[i]);
            if self.damped[i].abs() < 1e-20 {
                self.damped[i] = 0.0;
            }
            out[i] = self.damped[i] * self.gain[i];
        }
        let half = (out[0] + out[1] + out[2] + out[3]) * 0.5;
        for i in 0..4 {
            let n = self.lines[i].len();
            self.lines[i][self.at[i]] = out[i] - half + input * 0.5;
            self.at[i] = (self.at[i] + 1) % n;
        }
        [out[0] + out[2], out[1] + out[3]]
    }
}
/// Sound physics' echoes and what the sources sent them this buffer.
struct Reverb {
    rate: u32,
    echoes: [Echo; 4],
    input: [Vec<f32>; 4],
}
impl Reverb {
    fn new(rate: u32) -> Self {
        Self {
            rate,
            echoes: std::array::from_fn(|i| Echo::new(REVERB_DECAYS[i], REVERB_LINES[i], rate)),
            input: std::array::from_fn(|_| Vec::new()),
        }
    }
}
/// The one-pole low-pass coefficient that leaves `highs` of a 5 kHz tone (OpenAL EFX's
/// low-pass `GAINHF`); 1 passes everything.
fn muffle_coefficient(highs: f32, rate: u32) -> f32 {
    let highs = highs.clamp(1e-4, 1.0);
    if highs >= 0.999 {
        return 1.0;
    }
    let cutoff = 5_000.0 / (1.0 / (highs * highs) - 1.0).sqrt();
    1.0 - (-2.0 * std::f32::consts::PI * cutoff / rate.max(1) as f32).exp()
}
/// How much under water dulls every world sound.
const UNDERWATER_HIGHS: f32 = 0.2;

/// How one source is mixed into this buffer.
enum Voice {
    /// A stereo buffer, as OpenAL renders one: its channels as they are, at its gain
    /// (no panning, no distance attenuation).
    Stereo(f32),
    /// A mono source panned: `[left, right]` gains, and the gain alone for a mono output.
    Panned([f32; 2], f32),
    /// A mono source through Directional Audio's ears, at its gain.
    Binaural(f32, EarSettings),
}
// Only the output stream mixes; a build without one (no `audio` feature) never does.
#[cfg_attr(not(feature = "audio"), allow(dead_code))]
fn mix(mixer: &mut Mixer, data: &mut [f32], channels: usize, rate: u32) {
    data.fill(0.0);
    // The camera's forward and up vectors (`Camera.forwardVector`/`upVector`)
    // and OpenAL's right = forward x up.
    let basis = |yaw: f32, pitch: f32| {
        let (yaw, pitch) = (f64::from(yaw.to_radians()), f64::from(pitch.to_radians()));
        let forward = [
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        ];
        let up = [
            -yaw.sin() * pitch.sin(),
            pitch.cos(),
            yaw.cos() * pitch.sin(),
        ];
        let right = [
            forward[1] * up[2] - forward[2] * up[1],
            forward[2] * up[0] - forward[0] * up[2],
            forward[0] * up[1] - forward[1] * up[0],
        ];
        (forward, up, right)
    };
    // Every camera hearing (splitscreen: the other players' too), in the mixer's own buffer.
    let mut cameras = std::mem::take(&mut mixer.cameras);
    cameras.clear();
    cameras.extend(
        std::iter::once((
            0,
            (mixer.listener, mixer.listener_yaw, mixer.listener_pitch),
        ))
        .chain(
            mixer
                .listeners
                .iter()
                .map(|(&owner, &camera)| (owner, camera)),
        )
        .map(|(owner, (at, yaw, pitch))| {
            (
                at,
                basis(yaw, pitch),
                mixer.worlds.get(&owner).copied().unwrap_or_default(),
            )
        }),
    );
    let shared = cameras.len() > 1;
    // Several players on the speakers have no one head to render for.
    let binaural = mixer.directional && !shared && channels >= 2;
    let dot = |a: [f64; 3], b: [f64; 3]| (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) as f32;
    // Gains and pans per source for this buffer.
    let mut voices: Vec<(ChannelId, Voice, f64, f32, [f32; 4])> = Vec::new();
    let physics_on = mixer.physics;
    let underwater = mixer.underwater && physics_on;
    for (id, c) in &mixer.channels {
        let Some((_, source_rate, width)) = &c.samples else {
            continue;
        };
        if c.state != State::Playing {
            continue;
        }
        let at = [f64::from(c.at[0]), f64::from(c.at[1]), f64::from(c.at[2])];
        let away = |listener: [f64; 3]| {
            [
                at[0] - listener[0],
                at[1] - listener[1],
                at[2] - listener[2],
            ]
        };
        let length = |d: [f64; 3]| d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        // The nearest camera hears it (one camera but in splitscreen).
        let camera = cameras
            .iter()
            .filter(|camera| c.relative || camera.2 == c.world)
            .min_by(|a, b| length(away(a.0)).total_cmp(&length(away(b.0))))
            .map(|camera| (camera.0, camera.1));
        // Unheard sources still advance: returning to a world must not resume stale sounds.
        let (listener, (forward, up, right)) = camera.unwrap_or(([0.0; 3], basis(0.0, 0.0)));
        let d = if c.relative { at } else { away(listener) };
        let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() as f32;
        let gain = if camera.is_some() {
            c.gain.clamp(0.0, 1.0) * distance_gain(c.linear, distance)
        } else {
            0.0
        };
        // Relative sources are already in OpenAL's listener space (-Z ahead).
        let direction = if c.relative {
            [d[0] as f32, d[1] as f32, -d[2] as f32]
        } else {
            [dot(d, right), dot(d, up), dot(d, forward)]
        };
        let voice = if *width == 2 {
            Voice::Stereo(if camera.is_some() {
                c.gain.clamp(0.0, 1.0)
            } else {
                0.0
            })
        } else if binaural {
            Voice::Binaural(gain, ear_settings(direction, rate))
        } else {
            let [mut left, mut right] = stereo_gains(direction[0], direction[1], direction[2]);
            if shared {
                // Several players on the speakers: half the stereo width, as left and right
                // are nobody's in particular.
                let middle = (left + right) / 2.0;
                left = middle + (left - middle) / 2.0;
                right = middle + (right - middle) / 2.0;
            }
            Voice::Panned([left, right].map(|pan| gain * pan), gain)
        };
        let step = f64::from(*source_rate) / f64::from(rate) * f64::from(c.pitch.max(0.0));
        // Sound physics: a world sound muffled by what's between, and its echoes.
        let world_sound = physics_on && *width == 1 && !c.relative && c.linear.is_some();
        let (mut muffle, mut sends) = (1.0, [0.0; 4]);
        if world_sound {
            let mut highs = 1.0;
            if let Some((p, _)) = &c.physics {
                highs = p.highs;
                sends = p.sends.map(|g| g * gain);
            }
            if underwater {
                highs *= UNDERWATER_HIGHS;
                sends = sends.map(|g| g * 0.5);
            }
            muffle = muffle_coefficient(highs, rate);
        }
        let voice = match (voice, c.physics.as_ref().filter(|_| world_sound)) {
            (Voice::Panned(lr, g), Some((p, _))) => {
                Voice::Panned(lr.map(|x| x * p.gain), g * p.gain)
            }
            (Voice::Binaural(g, set), Some((p, _))) => Voice::Binaural(g * p.gain, set),
            (v, _) => v,
        };
        voices.push((*id, voice, step, muffle, sends));
    }
    let frames_out_count = data.len() / channels.max(1);
    let any_send = voices.iter().any(|v| v.4.iter().any(|g| *g > 0.0));
    if any_send && mixer.reverb.as_ref().is_none_or(|r| r.rate != rate) {
        mixer.reverb = Some(Box::new(Reverb::new(rate)));
    }
    if let Some(r) = &mut mixer.reverb {
        for input in &mut r.input {
            input.clear();
            input.resize(frames_out_count, 0.0);
        }
    }
    mixer.cameras = cameras;
    let frames_out = (data.len() / channels.max(1)).max(1);
    let mut reverb = mixer.reverb.take();
    for (id, voice, step, muffle, sends) in voices {
        let Some(c) = mixer.channels.get_mut(&id) else {
            continue;
        };
        let sending = sends.iter().any(|g| *g > 0.0);
        let Some((samples, _, width)) = c.samples.clone() else {
            continue;
        };
        let width = usize::from(width.max(1));
        let frames = samples.len() / width;
        // Directional Audio glides each ear's delay across the buffer, so a moving
        // source doesn't click.
        let (from, to) = match &voice {
            Voice::Binaural(_, set) => (c.ears.delay.unwrap_or(set.delay), set.delay),
            _ => ([0.0; 2], [0.0; 2]),
        };
        for (f, frame) in data.chunks_mut(channels).enumerate() {
            let mut i = c.position as usize;
            if i + 1 >= frames {
                if c.looping && frames > 1 {
                    c.position = 0.0;
                    i = 0;
                } else {
                    c.state = State::Stopped;
                    break;
                }
            }
            let t = (c.position - i as f64) as f32;
            let sample =
                |ch: usize| samples[i * width + ch] * (1.0 - t) + samples[(i + 1) * width + ch] * t;
            match &voice {
                Voice::Stereo(gain) => {
                    let (l, r) = (sample(0), sample(width - 1));
                    if channels >= 2 {
                        frame[0] += l * gain;
                        frame[1] += r * gain;
                    } else {
                        frame[0] += (l + r) / 2.0 * gain;
                    }
                }
                Voice::Panned([left, right], gain) => {
                    let mut s = sample(0);
                    if sending && let Some(r) = &mut reverb {
                        for (k, g) in sends.iter().enumerate() {
                            r.input[k][f] += s * g;
                        }
                    }
                    if muffle < 1.0 {
                        c.muffled += muffle * (s - c.muffled);
                        s = c.muffled;
                    }
                    if channels >= 2 {
                        frame[0] += s * left;
                        frame[1] += s * right;
                    } else {
                        // A mono output has no panning: the source's own gain.
                        frame[0] += s * gain;
                    }
                }
                Voice::Binaural(gain, set) => {
                    let along = (f + 1) as f32 / frames_out as f32;
                    let delay = [0, 1].map(|e| from[e] + (to[e] - from[e]) * along);
                    let mut s = sample(0);
                    if sending && let Some(r) = &mut reverb {
                        for (k, g) in sends.iter().enumerate() {
                            r.input[k][f] += s * g;
                        }
                    }
                    if muffle < 1.0 {
                        c.muffled += muffle * (s - c.muffled);
                        s = c.muffled;
                    }
                    let s = s * gain * BINAURAL_LEVEL;
                    let [l, r] = hear(&mut c.ears, s, delay, set);
                    frame[0] += l;
                    frame[1] += r;
                }
            }
            c.position += step;
        }
        if let Voice::Binaural(_, set) = &voice {
            c.ears.delay = Some(set.delay);
        }
    }
    // The echoes, every buffer while they ring (a tail keeps going after its sound stops).
    if let Some(r) = &mut reverb {
        for (f, frame) in data.chunks_mut(channels).enumerate() {
            let mut out = [0.0f32; 2];
            for (k, echo) in r.echoes.iter_mut().enumerate() {
                let [l, rr] = echo.step(r.input[k].get(f).copied().unwrap_or(0.0));
                out[0] += l;
                out[1] += rr;
            }
            if channels >= 2 {
                frame[0] += out[0] * REVERB_LEVEL;
                frame[1] += out[1] * REVERB_LEVEL;
            } else {
                frame[0] += (out[0] + out[1]) * 0.5 * REVERB_LEVEL;
            }
        }
    }
    if physics_on {
        mixer.reverb = reverb;
    }
    // Limit the whole buffer together, preserving waveform and stereo balance
    // instead of flattening each overloaded sample; release over 200 ms
    // (OpenAL Soft's output limiter, `ALC_OUTPUT_LIMITER_SOFT`, releases
    // over 0.2 s).
    let peak = data.iter().fold(1.0f32, |peak, s| peak.max(s.abs()));
    mixer.limiter_peak = mixer.limiter_peak.max(peak);
    let release = (-1.0 / (0.2 * rate as f32)).exp();
    for frame in data.chunks_mut(channels) {
        let gain = mixer.limiter_peak.recip();
        for s in frame {
            *s *= gain;
        }
        mixer.limiter_peak = (mixer.limiter_peak * release).max(peak);
    }
}
/// Why the game should not open an output, if it should not:
/// `HORIZON_NO_AUDIO`, or a virtual screen (`vscreen`), where agents and
/// harnesses run the game and must stay off the user's speakers.
/// `HORIZON_AUDIO=1` opens the output anyway (testing into a null sink).
#[cfg(any(feature = "audio", test))]
fn output_disabled(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> Option<String> {
    if var("HORIZON_NO_AUDIO").is_some() {
        return Some("audio off: HORIZON_NO_AUDIO is set".into());
    }
    if var("HORIZON_AUDIO").is_some_and(|v| v == "1") {
        return None;
    }
    let display = var("WAYLAND_DISPLAY")?;
    Path::new(&display)
        .components()
        .any(|c| c.as_os_str() == "vscreen")
        .then(|| "audio off on a virtual screen (vscreen); HORIZON_AUDIO=1 opens it".into())
}
/// An output stream on `device` in its default configuration, whatever its
/// sample format: the mixer works in f32 and converts.
#[cfg(feature = "audio")]
fn open_device(
    device: &cpal::Device,
    mixer: Arc<Mutex<Mixer>>,
) -> Result<(cpal::Stream, cpal::SupportedStreamConfig)> {
    use cpal::SampleFormat as F;
    let config = device.default_output_config()?;
    let stream = match config.sample_format() {
        F::F32 => build_stream::<f32>(device, &config, mixer.clone()),
        F::F64 => build_stream::<f64>(device, &config, mixer.clone()),
        F::I16 => build_stream::<i16>(device, &config, mixer.clone()),
        F::I32 => build_stream::<i32>(device, &config, mixer.clone()),
        F::I8 => build_stream::<i8>(device, &config, mixer.clone()),
        F::U16 => build_stream::<u16>(device, &config, mixer.clone()),
        F::U32 => build_stream::<u32>(device, &config, mixer.clone()),
        F::U8 => build_stream::<u8>(device, &config, mixer.clone()),
        other => Err(format!("unsupported sample format {other}").into()),
    }?;
    stream.play()?;
    if let Ok(mut m) = mixer.lock() {
        m.device_name = device.name().unwrap_or_default();
        m.device_lost = false;
    }
    Ok((stream, config))
}
/// Try one output device for `open`: skip it when it was already tried, log it
/// when it opens, and keep its error when it does not.
#[cfg(feature = "audio")]
fn try_open_device(
    device: &cpal::Device,
    mixer: Arc<Mutex<Mixer>>,
    tried: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> Option<cpal::Stream> {
    let name = device.name().unwrap_or_else(|_| "unnamed device".into());
    if tried.contains(&name) {
        return None;
    }
    tried.push(name.clone());
    match open_device(device, mixer) {
        Ok((stream, config)) => {
            log::info!(
                "Horizon: audio output {name} ({} Hz, {} channels, {})",
                config.sample_rate().0,
                config.channels(),
                config.sample_format()
            );
            Some(stream)
        }
        Err(e) => {
            errors.push(format!("{name}: {e}"));
            None
        }
    }
}
/// The name of the system's default output device.
#[cfg(feature = "audio")]
fn default_output_name() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|d| d.name().ok())
}
/// A cheap fingerprint of what `output_devices` can list. On Linux, the ALSA
/// cards: the rest are `cpal`'s fixed names. On macOS, every Core Audio
/// device's name, inputs included: telling which of them play opens each one
/// (about 90 ms a second with twenty devices), naming them takes 2 or 3 ms.
/// `None` elsewhere, where the list is always looked at.
#[cfg(feature = "audio")]
fn card_fingerprint() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/asound/cards").ok()
    }
    #[cfg(target_os = "macos")]
    {
        let devices = cpal::default_host().devices().ok()?;
        Some(
            devices
                .filter_map(|d| d.name().ok())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}
/// Whether the output device list should be looked at again: only the ALSA
/// cards can add or remove an entry, and `None` means there is no fingerprint,
/// so the list has to be checked as before.
#[cfg(any(feature = "audio", test))]
fn cards_changed(seen: Option<&str>, cards: Option<&str>) -> bool {
    match (seen, cards) {
        (Some(seen), Some(cards)) => seen != cards,
        _ => true,
    }
}
#[cfg(feature = "audio")]
fn build_stream<T: cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    mixer: Arc<Mutex<Mixer>>,
) -> Result<cpal::Stream> {
    let rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let mut scratch = Vec::new();
    let on_error = mixer.clone();
    let stream = device.build_output_stream(
        &config.config(),
        move |data: &mut [T], _| {
            scratch.resize(data.len(), 0.0f32);
            if let Ok(mut m) = mixer.lock() {
                mix(&mut m, &mut scratch, channels, rate);
            } else {
                scratch.fill(0.0);
            }
            for (out, s) in data.iter_mut().zip(&scratch) {
                *out = T::from_sample(*s);
            }
        },
        move |e| {
            log::warn!("Horizon: audio stream error: {e}");
            if matches!(e, cpal::StreamError::DeviceNotAvailable)
                && let Ok(mut m) = on_error.lock()
            {
                m.device_lost = true;
            }
        },
        None,
    )?;
    Ok(stream)
}
/// Available audio output device names.
#[cfg(feature = "audio")]
pub fn output_devices() -> Vec<String> {
    if output_disabled(|name| std::env::var_os(name)).is_some() {
        return Vec::new();
    }
    let host = cpal::default_host();
    let mut names = Vec::new();
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name()
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
    }
    names
}

#[cfg(not(feature = "audio"))]
pub fn output_devices() -> Vec<String> {
    Vec::new()
}

impl Audio {
    pub fn new(root: &Path) -> Self {
        Self::with_device(root, "")
    }
    pub fn with_device(root: &Path, device_name: &str) -> Self {
        Self::with_output(root, true, device_name)
    }
    pub fn with_registry(root: &Path, registry: SoundRegistry, device_name: &str) -> Self {
        Self::with_loaded_registry(root, registry, true, device_name)
    }
    /// No output device: the dedicated server.
    pub fn silent(root: &Path) -> Self {
        Self::with_output(root, false, "")
    }
    /// The output device asked for (`soundDevice`), empty for the system's default; the
    /// default plays when that one can't be opened, as in vanilla.
    pub fn device(&self) -> &str {
        &self.device_name
    }
    fn with_output(root: &Path, output: bool, target_device: &str) -> Self {
        let registry = SoundRegistry::load(root);
        Self::with_loaded_registry(root, registry, output, target_device)
    }
    fn with_loaded_registry(
        root: &Path,
        registry: SoundRegistry,
        output: bool,
        target_device: &str,
    ) -> Self {
        if output && registry.is_empty() {
            // The game then plays nothing on a working output (#1035, #1076).
            log::warn!(
                "Horizon: no sounds in {}: they were not copied from your Minecraft \
                 install; prepare the game again in the launcher (or run \
                 horizon-prepare sounds)",
                root.join("assets").display()
            );
        }
        let mixer = Arc::new(Mutex::new(Mixer::default()));
        let opened = if output {
            Self::open_named(mixer.clone(), target_device)
        } else {
            Err("no output on a dedicated server".into())
        };
        let (stream, live) = match opened {
            Ok(v) => (Some(v), true),
            Err(e) => {
                if output {
                    log::warn!("Horizon: no sound: {e}");
                }
                (None, false)
            }
        };
        let mut engine = SoundEngine::new(registry);
        engine.loaded = live;
        Self {
            _stream: stream,
            device_name: target_device.to_owned(),
            engine,
            backend: MixerBackend {
                mixer,
                root: root.to_owned(),
                cache: Arc::default(),
                decoder: None,
                next: 0,
                live,
                owner: 0,
                world: 0,
                shares: Arc::default(),
            },
            record_heard: false,
            listener: [0.0; 3],
            listener_yaw: 0.0,
            master: 1.0,
            random: LegacyRandom::new(crate::sound_engine::unique_seed()),
            source_override: None,
            capture: None,
            heard: Default::default(),
            stopped: Default::default(),
            preview: Default::default(),
            #[cfg(feature = "audio")]
            default_seen: None,
            #[cfg(feature = "audio")]
            cards_seen: None,
            #[cfg(feature = "audio")]
            devices_seen: Vec::new(),
            #[cfg(feature = "audio")]
            device_probe_started: std::time::Instant::now(),
            #[cfg(feature = "audio")]
            device_probe: None,
        }
    }
    #[cfg(not(feature = "audio"))]
    fn open_named(_: Arc<Mutex<Mixer>>, _: &str) -> Result<Stream> {
        Err("this build has no sound output (built without the `audio` feature)".into())
    }
    #[cfg(feature = "audio")]
    fn open_named(mixer: Arc<Mutex<Mixer>>, target: &str) -> Result<Stream> {
        if let Some(reason) = output_disabled(|name| std::env::var_os(name)) {
            return Err(reason.into());
        }
        let host = cpal::default_host();
        if !target.is_empty()
            && let Ok(devices) = host.output_devices()
        {
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "unnamed device".into());
                if name == target {
                    match open_device(&device, mixer.clone()) {
                        Ok((stream, config)) => {
                            log::info!(
                                "Horizon: audio output {name} ({} Hz, {} channels, {})",
                                config.sample_rate().0,
                                config.channels(),
                                config.sample_format()
                            );
                            return Ok(stream);
                        }
                        Err(e) => {
                            log::warn!(
                                "Horizon: could not open audio device '{target}': {e}; falling back to default"
                            );
                        }
                    }
                    break;
                }
            }
        }
        Self::open(mixer)
    }
    #[cfg(feature = "audio")]
    pub fn set_device(&mut self, name: &str) -> Result<()> {
        if !self.backend.live && self._stream.is_none() && !self.engine.loaded {
            self.device_name = name.to_owned();
            return Ok(());
        }
        self._stream = Some(Self::open_named(self.backend.mixer.clone(), name)?);
        self.device_name = name.to_owned();
        self.backend.live = true;
        self.engine.loaded = true;
        Ok(())
    }
    #[cfg(not(feature = "audio"))]
    pub fn set_device(&mut self, name: &str) -> Result<()> {
        self.device_name = name.to_owned();
        Ok(())
    }
    #[cfg(feature = "audio")]
    fn open(mixer: Arc<Mutex<Mixer>>) -> Result<Stream> {
        if let Some(reason) = output_disabled(|name| std::env::var_os(name)) {
            return Err(reason.into());
        }
        let host = cpal::default_host();
        let mut errors = Vec::new();
        // The default device first: it is what plays, and listing the others
        // probes every PCM, which is slow and prints ALSA's errors for the
        // plugins this system lacks.
        let mut tried = Vec::new();
        if let Some(device) = host.default_output_device()
            && let Some(stream) = try_open_device(&device, mixer.clone(), &mut tried, &mut errors)
        {
            return Ok(stream);
        }
        // A default that cannot open (busy, or a stale ALSA default) should not
        // leave the game silent when another device works.
        for device in host.output_devices().into_iter().flatten() {
            if let Some(stream) = try_open_device(&device, mixer.clone(), &mut tried, &mut errors) {
                return Ok(stream);
            }
        }
        if errors.is_empty() {
            return Err(format!("no audio output device on the {} host", host.id().name()).into());
        }
        Err(format!("no audio output device opened ({})", errors.join("; ")).into())
    }
    fn bound(&mut self) -> Bound<'_, MixerBackend> {
        Bound {
            engine: &mut self.engine,
            backend: &mut self.backend,
        }
    }
    /// Sound physics on or off, and whether the listener is under water. Off, the echoes go and
    /// every sound plays as vanilla's.
    pub fn set_sound_physics(&self, on: bool, underwater: bool) {
        if let Ok(mut m) = self.backend.mixer.lock() {
            if !on && m.physics {
                m.reverb = None;
                for c in m.channels.values_mut() {
                    c.physics = None;
                }
            }
            m.physics = on;
            m.underwater = underwater;
        }
    }
    /// World sounds sound physics hasn't measured yet, or measured longer than `again` ago
    /// and still playing on a loop: their channels and where they are.
    pub fn physics_wanted(&self, again: std::time::Duration) -> Vec<(ChannelId, [f64; 3])> {
        let Ok(m) = self.backend.mixer.lock() else {
            return Vec::new();
        };
        if !m.physics {
            return Vec::new();
        }
        let now = std::time::Instant::now();
        m.channels
            .iter()
            .filter(|(_, c)| {
                !c.relative
                    && c.linear.is_some()
                    && matches!(c.state, State::Initial | State::Playing)
                    && match &c.physics {
                        None => true,
                        Some((_, at)) => c.looping && now.duration_since(*at) >= again,
                    }
            })
            .map(|(id, c)| {
                (
                    *id,
                    [f64::from(c.at[0]), f64::from(c.at[1]), f64::from(c.at[2])],
                )
            })
            .collect()
    }
    /// What sound physics measured for each channel.
    pub fn set_physics(&self, measured: Vec<(ChannelId, Physics)>) {
        if let Ok(mut m) = self.backend.mixer.lock() {
            let now = std::time::Instant::now();
            for (id, p) in measured {
                if let Some(c) = m.channels.get_mut(&id) {
                    c.physics = Some((p, now));
                }
            }
        }
    }
    /// `options.directionalAudio`: render mono sources for headphones (`ear_settings`).
    pub fn set_directional(&mut self, on: bool) {
        if let Ok(mut m) = self.backend.mixer.lock() {
            m.directional = on;
        }
    }
    /// World identity for this player's sources and camera. Existing channels keep their world.
    pub fn set_world(&mut self, world: &str) {
        if let Ok(mut mixer) = self.backend.mixer.lock() {
            let id = mixer.world_id(world);
            self.backend.world = id;
            mixer.worlds.insert(self.backend.owner, id);
        }
    }

    /// `SoundManager.updateSource`: the camera position and rotation
    /// (`Listener.setTransform` takes its forward and up vectors).
    pub fn set_listener(&mut self, position: [f64; 3], yaw: f32, pitch: f32) {
        self.listener = position;
        self.listener_yaw = yaw;
        if let Ok(mut m) = self.backend.mixer.lock() {
            if self.backend.owner == 0 {
                m.listener = position;
                m.listener_yaw = yaw;
                m.listener_pitch = pitch;
            } else {
                m.listeners
                    .insert(self.backend.owner, (position, yaw, pitch));
            }
        }
    }
    /// Splitscreen: another player's sounds on this output, heard from its camera too
    /// (`owner`: 1 to 255, its own). Dropped, its camera and sounds go.
    pub fn share(&self, owner: u8) -> Self {
        assert_ne!(owner, 0, "shared audio owners must be nonzero");
        self.backend
            .shares
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut engine = SoundEngine::new(self.engine.registry.clone());
        engine.loaded = self.backend.live;
        Self {
            _stream: None,
            device_name: self.device_name.clone(),
            engine,
            backend: MixerBackend {
                mixer: self.backend.mixer.clone(),
                root: self.backend.root.clone(),
                cache: self.backend.cache.clone(),
                decoder: None,
                next: (SHARED_CHANNELS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1)
                    << 40,
                live: self.backend.live,
                owner,
                world: self.backend.world,
                shares: self.backend.shares.clone(),
            },
            record_heard: false,
            listener: [0.0; 3],
            listener_yaw: 0.0,
            master: self.master,
            random: LegacyRandom::new(crate::sound_engine::unique_seed()),
            source_override: None,
            capture: None,
            heard: Default::default(),
            stopped: Default::default(),
            preview: Default::default(),
            // The device belongs to the first player's output, which looks after it.
            #[cfg(feature = "audio")]
            default_seen: self.default_seen.clone(),
            #[cfg(feature = "audio")]
            cards_seen: self.cards_seen.clone(),
            #[cfg(feature = "audio")]
            devices_seen: self.devices_seen.clone(),
            #[cfg(feature = "audio")]
            device_probe_started: std::time::Instant::now(),
            #[cfg(feature = "audio")]
            device_probe: None,
        }
    }
    /// Sync the volume options, refreshing the categories that changed
    /// (the option sliders' `refreshCategoryVolume`).
    fn sync_volumes(&mut self) {
        let mut volumes = [self.master; 11];
        for (i, v) in volumes.iter_mut().enumerate().skip(1) {
            *v = crate::settings::source_volume(i);
        }
        for source in 0..11 {
            if self.engine.source_volumes[source] != volumes[source] {
                self.engine.source_volumes[source] = volumes[source];
                self.engine
                    .refresh_category_volume(source, &mut self.backend);
            }
        }
        self.engine.listening = self.record_heard;
    }
    /// Start one device query without ever making the audio/render thread wait
    /// for CPAL or the operating system. At most one query is alive per `Audio`.
    #[cfg(feature = "audio")]
    fn start_device_probe(&self) -> Option<std::sync::mpsc::Receiver<DeviceProbe>> {
        let previous_default = self.default_seen.clone();
        let previous_cards = self.cards_seen.clone();
        let previous_devices = self.devices_seen.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        match std::thread::Builder::new()
            .name("Audio device probe".into())
            .spawn(move || {
                let default = default_output_name();
                let cards = card_fingerprint();
                let devices = if default == previous_default
                    && !cards_changed(previous_cards.as_deref(), cards.as_deref())
                {
                    previous_devices
                } else {
                    output_devices()
                };
                let _ = sender.send((default, cards, devices));
            }) {
            Ok(_) => Some(receiver),
            Err(error) => {
                log::warn!("Horizon: could not start the audio device probe: {error}");
                None
            }
        }
    }
    /// `SoundEngine.shouldChangeDevice`: the open device was lost, the system
    /// default changed (with no device chosen in the options), or the chosen
    /// device came back; the output is then opened again. Device discovery runs
    /// off-thread once a second; this function only polls its completed result.
    #[cfg(feature = "audio")]
    fn should_change_device(&mut self) -> bool {
        // A splitscreen share plays through the first player's output: it leaves the device,
        // and the mixer's lost flag, to that output.
        if self.backend.owner != 0 {
            return false;
        }
        let (current, lost) = match self.backend.mixer.lock() {
            Ok(mut m) => (m.device_name.clone(), std::mem::take(&mut m.device_lost)),
            Err(_) => return false,
        };
        if lost {
            log::info!("Horizon: audio device was lost");
            self.default_seen = None;
            self.devices_seen.clear();
            self.device_probe_started = std::time::Instant::now();
            return true;
        }
        let mut found = None;
        if let Some(result) = self.device_probe.as_ref().map(|probe| probe.try_recv()) {
            match result {
                Ok(value) => {
                    found = Some(value);
                    self.device_probe = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.device_probe = None,
            }
        }
        let mut changed = false;
        if let Some((default, cards, devices)) = found {
            self.cards_seen = cards;
            if default != self.default_seen || devices != self.devices_seen {
                // Only a change in the device lists is looked at, as in vanilla: a
                // preferred device that is listed but cannot open is not retried.
                self.default_seen = default.clone();
                let wanted = self.device_name.clone();
                let available = devices.contains(&wanted);
                self.devices_seen = devices;
                if wanted.is_empty() {
                    changed = default.as_deref().is_some_and(|d| d != current);
                    if changed {
                        log::info!("Horizon: system default audio device has changed");
                    }
                } else {
                    changed = current != wanted && available;
                    if changed {
                        log::info!("Horizon: preferred audio device has become available");
                    }
                }
            }
        }
        #[cfg(not(target_os = "android"))]
        if self._stream.is_some()
            && self.device_probe.is_none()
            && self.device_probe_started.elapsed() >= std::time::Duration::from_secs(1)
        {
            self.device_probe_started = std::time::Instant::now();
            self.device_probe = self.start_device_probe();
        }
        changed
    }
    /// `SoundManager.tick(paused)`, once per client tick.
    pub fn tick(&mut self, paused: bool, ctx: &TickContext) {
        #[cfg(feature = "audio")]
        {
            if self.should_change_device() {
                // SoundEngine.reload: stop every sound, open the output again.
                self.stop_all();
                let device = self.device_name.clone();
                if let Err(e) = self.set_device(&device) {
                    log::warn!("Horizon: could not reopen the audio output: {e}");
                }
            }
        }
        self.sync_volumes();
        self.engine.tick(paused, ctx, &mut self.backend);
    }
    /// `SoundManager.pauseAllExcept(MUSIC, UI)`.
    pub fn pause(&mut self) {
        self.engine.pause_all_except(
            &[crate::sound_engine::MUSIC, crate::sound_engine::UI],
            &mut self.backend,
        );
    }
    /// `SoundManager.resume`.
    pub fn resume(&mut self) {
        self.engine.resume(&mut self.backend);
    }
    /// `SoundManager.stop()`: every sound.
    pub fn stop_all(&mut self) {
        self.engine.stop_all(&mut self.backend);
    }
    /// `SoundManager.reload` (resource packs changed): stop every sound, then
    /// read sound events and sound files from the new resource root.
    pub fn reload(&mut self, root: &Path) {
        self.stop_all();
        self.engine.registry = SoundRegistry::load(root);
        self.backend.root = root.to_owned();
        let mut cache = self.backend.cache.lock().unwrap();
        cache.entries.clear();
        cache.bytes = 0;
    }
    /// Play a sound event at a world position, as `ClientLevel.playLocalSound`
    /// (a `SimpleSoundInstance` seeded from the level random), or relative to
    /// the listener without attenuation (`None`).
    pub fn play(&mut self, event: &str, at: Option<[f64; 3]>, volume: f32, pitch: f32) {
        let seed = self.random.next_long();
        self.play_seeded(event, at, volume, pitch, seed);
    }
    /// `ClientLevel.playLocalSound` with `distanceDelay`: a sound more than 10
    /// blocks from the camera starts `sqrt(distance) / 40` seconds later.
    pub fn play_distance_delayed(
        &mut self,
        event: &str,
        at: [f64; 3],
        volume: f32,
        pitch: f32,
        camera: [f64; 3],
    ) {
        let distance_sqr: f64 = (0..3).map(|i| (at[i] - camera[i]).powi(2)).sum();
        if distance_sqr <= 100.0 {
            self.play(event, Some(at), volume, pitch);
            return;
        }
        self.play_after(
            event,
            at,
            volume,
            pitch,
            (distance_sqr.sqrt() / 40.0 * 20.0) as i32,
        );
    }
    /// `play` at a world position, starting `ticks` sound-engine ticks (20 a
    /// second) from now: `SoundEngine.playDelayed`.
    pub fn play_after(&mut self, event: &str, at: [f64; 3], volume: f32, pitch: f32, ticks: i32) {
        let seed = self.random.next_long();
        if let Some(capture) = self.capture.as_mut() {
            capture.push((event.to_owned(), at, volume, pitch));
        }
        *self.heard.entry(event.to_owned()).or_default() += 1;
        self.sync_volumes();
        let event = crate::sound_events::normalize(event);
        let source = self
            .source_override
            .unwrap_or_else(|| crate::settings::sound_source(&event));
        let instance =
            SoundInstance::at(&event, source, volume, pitch, LegacyRandom::new(seed), at);
        let id = self.engine.add(instance, false);
        self.engine.play_delayed(id, ticks);
    }
    /// `play` for this client only: a sound a packet's handler plays
    /// (`playLocalSound`), which the guests play from that packet themselves.
    pub fn play_unshared(&mut self, event: &str, at: Option<[f64; 3]>, volume: f32, pitch: f32) {
        let capture = self.capture.take();
        self.play(event, at, volume, pitch);
        self.capture = capture;
    }
    /// `AbstractWidget.playButtonClickSound`: an unseeded, relative UI sound.
    pub fn play_button_click(&mut self) {
        self.play_instance(
            SoundInstance::for_ui(
                "minecraft:ui.button.click",
                1.0,
                0.25,
                SoundEngine::unseeded(),
            ),
            false,
        );
    }
    /// `SoundPreviewHandler.preview`: a category volume slider moved to
    /// `volume` on the title screen's options.
    pub fn preview(&mut self, source: usize, volume: f32) {
        let mut preview = std::mem::take(&mut self.preview);
        preview.preview(self, source, volume);
        self.preview = preview;
    }
    /// `play` with the seed a sound packet carries.
    pub fn play_seeded(
        &mut self,
        event: &str,
        at: Option<[f64; 3]>,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) {
        if let (Some(capture), Some(at)) = (self.capture.as_mut(), at) {
            capture.push((event.to_owned(), at, volume, pitch));
        }
        if at.is_some() {
            *self.heard.entry(event.to_owned()).or_default() += 1;
        }
        self.sync_volumes();
        let event = crate::sound_events::normalize(event);
        let source = self
            .source_override
            .unwrap_or_else(|| crate::settings::sound_source(&event));
        let random = LegacyRandom::new(seed);
        let instance = match at {
            Some(at) => SoundInstance::at(&event, source, volume, pitch, random, at),
            None => SoundInstance::new(
                &event,
                source,
                volume,
                pitch,
                random,
                false,
                0,
                Attenuation::None,
                [0.0; 3],
                true,
            ),
        };
        self.play_instance(instance, false);
    }
    /// Play a sound on a given `SoundSource` index (`/playsound`).
    pub fn play_on(
        &mut self,
        source: usize,
        event: &str,
        at: Option<[f64; 3]>,
        volume: f32,
        pitch: f32,
    ) {
        self.source_override = Some(source);
        self.play(event, at, volume, pitch);
        self.source_override = None;
    }
    /// `SimpleSoundInstance.forJukeboxSong` at a jukebox; the handle stays
    /// until `release`.
    pub fn play_jukebox_song(&mut self, event: &str, pos: [i32; 3]) -> InstanceId {
        let at = [
            f64::from(pos[0]) + 0.5,
            f64::from(pos[1]) + 0.5,
            f64::from(pos[2]) + 0.5,
        ];
        let event = crate::sound_events::normalize(event);
        let instance = SoundInstance::for_jukebox_song(&event, at, SoundEngine::unseeded());
        self.play_instance(instance, true).0
    }
    /// Stop and drop a held instance.
    pub fn stop_and_release(&mut self, id: InstanceId) {
        self.stop_instance(id);
        self.release(id);
    }
    /// `SoundBufferLibrary.enumerate`: the decoded buffers kept and their bytes.
    pub fn cache_stats(&self) -> (usize, usize) {
        self.backend
            .cache
            .lock()
            .map_or((0, 0), |c| (c.entries.len(), c.bytes))
    }
    /// Every sound event id (for command completion).
    /// The engine's table sizes and the mixer's channels (`status`).
    pub fn census(&self) -> serde_json::Value {
        let mut census = self.engine.census();
        census["channels"] = self
            .backend
            .mixer
            .lock()
            .map_or(0, |m| m.channels.len())
            .into();
        if let Ok(cache) = self.backend.cache.lock() {
            census["decoded"] = cache.entries.len().into();
            census["decoded_mib"] = (cache.bytes >> 20).into();
        }
        census
    }
    pub fn event_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.engine.registry.names().cloned().collect();
        v.sort();
        v
    }
    /// `ClientboundStopSoundPacket`: stop the playing sounds of an event
    /// and/or a source.
    pub fn stop_sounds(&mut self, event: Option<&str>, source: Option<usize>) {
        let event = event.map(crate::sound_events::normalize);
        let selection = format!(
            "{} {}",
            source
                .and_then(|s| crate::settings::SOUND_SOURCES.get(s))
                .map_or("*", |s| s),
            event.as_deref().unwrap_or("*")
        );
        *self.stopped.entry(selection).or_default() += 1;
        self.engine
            .stop_matching(event.as_deref(), source, &mut self.backend);
    }
    /// The sounds played since the last call, for the subtitle overlay.
    pub fn take_heard(&mut self) -> Vec<Heard> {
        std::mem::take(&mut self.engine.notifications)
            .into_iter()
            .filter_map(|n| {
                Some(Heard {
                    subtitle: n.subtitle?,
                    at: Some(n.position),
                    range: n.range,
                    looping: n.looping,
                })
            })
            .collect()
    }
}
impl Sounds for Audio {
    fn engine(&mut self) -> &mut SoundEngine {
        &mut self.engine
    }
    fn play_instance(&mut self, instance: SoundInstance, retain: bool) -> (InstanceId, PlayResult) {
        self.sync_volumes();
        self.bound().play_instance(instance, retain)
    }
    fn play_id(&mut self, id: InstanceId) -> PlayResult {
        self.sync_volumes();
        self.bound().play_id(id)
    }
    fn stop_instance(&mut self, id: InstanceId) {
        self.bound().stop_instance(id);
    }
    fn update_category_volume(&mut self, source: usize, gain: f32) {
        self.bound().update_category_volume(source, gain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_click_uses_vanilla_ui_parameters_without_advancing_world_random() {
        let mut audio = Audio::silent(Path::new("/nonexistent/horizon-test-assets"));
        audio.engine.registry.add_file(
            "minecraft",
            br#"{"ui.button.click":{"sounds":["gui/button"]}}"#,
            &|_, _| true,
        );
        // Exercise the mixer without opening an output device. The missing
        // sample is harmless: this test checks the source the engine creates.
        audio.engine.loaded = true;
        audio.backend.live = true;
        let mut random = audio.random.clone();
        audio.play_button_click();
        let instance = audio.engine.instance(1).expect("click reached the engine");
        assert_eq!(instance.identifier, "minecraft:ui.button.click");
        assert_eq!(instance.source, crate::sound_engine::UI);
        assert_eq!(instance.volume, 0.25);
        assert_eq!(instance.pitch, 1.0);
        assert_eq!(instance.attenuation, Attenuation::None);
        assert_eq!(instance.position, [0.0; 3]);
        assert!(instance.relative);
        assert!(!instance.looping);
        assert_eq!(audio.random.next_long(), random.next_long());
        audio.pause();
        assert_eq!(
            audio.backend.mixer.lock().unwrap().channels[&1].state,
            State::Initial
        );
        audio.stop_all();
    }

    fn channel(samples: Vec<f32>, linear: Option<f32>, at: [f32; 3], looping: bool) -> Channel {
        Channel {
            samples: Some((Arc::new(samples), 48_000, 1)),
            position: 0.0,
            state: State::Playing,
            gain: 1.0,
            pitch: 1.0,
            linear,
            looping,
            at,
            relative: false,
            owner: 0,
            world: 0,
            started: None,
            ears: Ears::default(),
            physics: None,
            muffled: 0.0,
        }
    }

    #[test]
    fn decoded_sounds_stay_within_the_budget_least_recently_used_out() {
        let mut cache = SoundCache::default();
        let quarter = CACHE_BYTES / 4 / std::mem::size_of::<f32>();
        let sound = || Some((Arc::new(vec![0.0f32; quarter]), 44100, 1));
        for key in ["a", "b", "c", "d"] {
            cache.insert(key.into(), sound());
        }
        assert!(cache.get("a").is_some());
        cache.insert("e".into(), sound());
        assert!(cache.bytes <= CACHE_BYTES);
        assert!(
            cache.get("b").is_none(),
            "the least recently used goes first"
        );
        assert!(cache.get("a").is_some() && cache.get("e").is_some());
        // A failed load is remembered without counting.
        cache.insert("missing".into(), None);
        assert_eq!(cache.get("missing"), Some(None));
    }
    #[test]
    fn output_stays_off_on_a_virtual_screen_unless_asked() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |name: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| std::ffi::OsString::from(*v))
            }
        };
        assert!(output_disabled(env(&[("WAYLAND_DISPLAY", "wayland-1")])).is_none());
        assert!(output_disabled(env(&[])).is_none());
        const VSCREEN: &str = "/run/user/1000/vscreen/main/wayland-1";
        assert!(output_disabled(env(&[("WAYLAND_DISPLAY", VSCREEN)])).is_some());
        assert!(
            output_disabled(env(&[("WAYLAND_DISPLAY", VSCREEN), ("HORIZON_AUDIO", "1")])).is_none()
        );
        assert!(
            output_disabled(env(&[("HORIZON_NO_AUDIO", "1"), ("HORIZON_AUDIO", "1")])).is_some()
        );
    }
    #[test]
    fn linear_attenuation_reaches_zero_at_the_max_distance() {
        assert_eq!(distance_gain(None, 100.0), 1.0);
        assert_eq!(distance_gain(Some(16.0), 0.0), 1.0);
        assert_eq!(distance_gain(Some(16.0), 8.0), 0.5);
        assert_eq!(distance_gain(Some(16.0), 16.0), 0.0);
        assert_eq!(distance_gain(Some(16.0), 40.0), 0.0);
    }

    #[test]
    fn overlapping_voices_are_limited_without_flattening_or_changing_stereo_balance() {
        let mut m = Mixer::default();
        let wave = vec![0.25, 0.5, 0.75, 1.0, -1.0, -0.75, -0.5, -0.25, 0.0];
        for id in 1..=4 {
            m.channels
                .insert(id, channel(wave.clone(), None, [-1.0, 0.0, 0.0], false));
        }
        let mut data = [0.0; 16];
        mix(&mut m, &mut data, 2, 48_000);
        // West of a listener facing south: fully on the right.
        for (frame, sample) in data.as_chunks::<2>().0.iter().zip(wave) {
            assert!(frame[0].abs() < 1e-6, "{frame:?}");
            assert!((frame[1] - sample).abs() < 1e-6, "{frame:?}");
        }
        assert!(data.iter().all(|s| s.abs() <= 1.0));
    }

    #[test]
    fn limiter_leaves_quiet_audio_alone_and_recovers_across_callbacks() {
        let mut m = Mixer::default();
        m.channels
            .insert(1, channel(vec![0.25; 2], None, [0.0; 3], true));
        let quiet = 0.25 * stereo_gains(0.0, 0.0, 1.0)[0];
        let mut data = [0.0; 16];
        mix(&mut m, &mut data, 2, 48_000);
        assert_eq!(data, [quiet; 16]);

        for id in 2..=5 {
            m.channels
                .insert(id, channel(vec![1.0; 2], None, [0.0; 3], true));
        }
        mix(&mut m, &mut data, 2, 48_000);
        assert!(data.iter().all(|s| s.abs() <= 1.0));
        m.channels.retain(|&id, _| id == 1);
        let mut recovery = vec![0.0; 2 * 48_000];
        mix(&mut m, &mut recovery, 2, 48_000);
        assert!(recovery[0] > 0.0 && recovery[0] < 0.1);
        assert!(recovery.windows(2).all(|w| w[1] >= w[0]));
        assert_eq!(*recovery.last().unwrap(), quiet);
    }

    #[test]
    fn sources_play_attenuated_then_stop_or_loop() {
        let mut m = Mixer::default();
        m.channels
            .insert(1, channel(vec![1.0; 4], Some(16.0), [8.0, 0.0, 0.0], false));
        m.channels
            .insert(2, channel(vec![1.0; 4], None, [0.0; 3], true));
        let mut data = [0.0f32; 16];
        mix(&mut m, &mut data, 2, 48_000);
        assert_eq!(m.channels[&1].state, State::Stopped);
        assert_eq!(m.channels[&2].state, State::Playing);
        // Mono sum of the two at the first frame: the far one is half as loud.
        let first = data[0] + data[1];
        assert!(first > 1.4 && first < 3.1, "{first}");
    }

    #[test]
    fn stereo_panning_matches_openal_soft() {
        // OpenAL Soft 1.25.2 (LWJGL 3.4.3's macOS build), stereo loopback
        // device with vanilla's attributes, 1 kHz sine 4 blocks away.
        let cases = [
            ((0.0, 0.0, 1.0), [0.5957, 0.5957]),
            ((-1.0, 0.0, 0.0), [1.0, 0.0]),
            ((1.0, 0.0, 0.0), [0.0, 1.0]),
            ((0.0, 0.0, -1.0), [0.4043, 0.4043]),
            ((0.0, 1.0, 0.0), [0.5, 0.5]),
            ((0.0, -1.0, 0.0), [0.5, 0.5]),
            ((-1.0, 0.0, 1.0), [0.9985, 0.0747]),
            ((-1.0, 0.0, -1.0), [0.7859, 0.0788]),
            ((0.0, 0.0, 0.0), [0.5957, 0.5957]),
        ];
        for ((right, up, forward), expected) in cases {
            let gains = stereo_gains(right, up, forward);
            for (g, e) in gains.into_iter().zip(expected) {
                assert!((g - e).abs() < 1e-4, "{right} {up} {forward}: {gains:?}");
            }
        }
    }

    #[test]
    fn panning_follows_the_camera_rotation() {
        let gains = |yaw: f32, pitch: f32, at: [f32; 3]| {
            let mut m = Mixer {
                listener_yaw: yaw,
                listener_pitch: pitch,
                ..Mixer::default()
            };
            m.channels.insert(1, channel(vec![1.0; 4], None, at, true));
            let mut data = [0.0; 2];
            mix(&mut m, &mut data, 2, 48_000);
            data
        };
        let close =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4;
        // Facing south (yaw 0), east is on the left; facing north, on the right.
        assert!(close(gains(0.0, 0.0, [1.0, 0.0, 0.0]), [1.0, 0.0]));
        assert!(close(gains(180.0, 0.0, [1.0, 0.0, 0.0]), [0.0, 1.0]));
        // Looking straight down, a source below is ahead and one south is above.
        assert!(close(gains(0.0, 90.0, [0.0, -1.0, 0.0]), [0.5957, 0.5957]));
        assert!(close(gains(0.0, 90.0, [0.0, 0.0, 1.0]), [0.5, 0.5]));
    }

    #[test]
    fn splitscreen_sounds_are_heard_from_the_nearest_camera() {
        // P1 far away, a second player beside the source: heard at full gain, centred.
        let mut m = Mixer {
            listener: [100.0, 0.0, 0.0],
            ..Mixer::default()
        };
        m.listeners.insert(1, ([0.0, 0.0, 1.0], 0.0, 0.0));
        m.channels
            .insert(1, channel(vec![1.0; 4], Some(16.0), [0.0, 0.0, 2.0], true));
        let mut data = [0.0; 2];
        mix(&mut m, &mut data, 2, 48_000);
        assert!(
            data[0] > 0.5 && (data[0] - data[1]).abs() < 1e-4,
            "{data:?}"
        );
        // Alone, P1 would not hear it: 100 blocks past its 16.
        m.listeners.clear();
        let mut data = [0.0; 2];
        mix(&mut m, &mut data, 2, 48_000);
        assert_eq!(data, [0.0; 2]);
    }

    #[test]
    fn a_shared_listener_registers_even_in_the_hosts_world() {
        let mut host = Audio::silent(Path::new("/nonexistent/horizon-test-audio"));
        host.set_world("minecraft:overworld");
        let mut guest = host.share(1);
        guest.set_world("minecraft:overworld");
        guest.set_listener([1.0, 2.0, 3.0], 0.0, 0.0);
        let id = guest.backend.create(false).unwrap();
        let mixer = host.backend.mixer.lock().unwrap();
        let overworld = mixer
            .world_names
            .iter()
            .position(|w| w == "minecraft:overworld");
        let overworld = overworld.map(|i| i as u32 + 1);
        assert_eq!(mixer.worlds.get(&1).copied(), overworld);
        assert_eq!(Some(mixer.channels[&id].world), overworld);
    }

    #[test]
    fn cameras_in_other_dimensions_cannot_hear_a_source() {
        let mut m = Mixer {
            listener: [100.0, 0.0, 0.0],
            ..Default::default()
        };
        m.listeners.insert(1, ([0.0, 0.0, 1.0], 0.0, 0.0));
        let nether = m.world_id("nether");
        m.worlds.insert(1, nether);
        m.channels
            .insert(1, channel(vec![1.0; 4], Some(16.0), [0.0, 0.0, 2.0], true));
        let mut data = [0.0; 2];
        mix(&mut m, &mut data, 2, 48_000);
        assert_eq!(
            data, [0.0; 2],
            "nearby Nether camera must not hear Overworld source"
        );
        m.channels.get_mut(&1).unwrap().world = nether;
        mix(&mut m, &mut data, 2, 48_000);
        assert!(data[0] > 0.5, "same-world camera must hear source");
        let end = m.world_id("end");
        m.worlds.insert(1, end);
        data.fill(0.0);
        mix(&mut m, &mut data, 2, 48_000);
        assert_eq!(data, [0.0; 2], "no listener remains in source's world");
    }

    #[test]
    fn sounds_in_a_world_without_listeners_finish_while_inaudible() {
        let mut mixer = Mixer::default();
        let mut source = channel(vec![1.0; 4], None, [0.0; 3], false);
        let nether = mixer.world_id("nether");
        source.world = nether;
        mixer.channels.insert(1, source);
        let mut data = [0.0; 4];
        mix(&mut mixer, &mut data, 2, 48_000);
        assert_eq!(data, [0.0; 4]);
        assert!(mixer.channels[&1].position > 0.0);
        mix(&mut mixer, &mut data, 2, 48_000);
        assert_eq!(mixer.channels[&1].state, State::Stopped);
        mixer.worlds.insert(0, nether);
        mix(&mut mixer, &mut data, 2, 48_000);
        assert_eq!(data, [0.0; 4], "returning must not resume the old one-shot");
    }

    #[test]
    fn variants_of_one_sound_are_the_same_sound() {
        assert_eq!(
            variantless("minecraft:mob/creeper/say1.ogg"),
            "minecraft:mob/creeper/say"
        );
        assert_eq!(
            variantless("minecraft:step/grass6.ogg"),
            "minecraft:step/grass"
        );
        assert_eq!(
            variantless("minecraft:random/click.ogg"),
            "minecraft:random/click"
        );
    }

    #[test]
    fn another_players_copy_of_a_sound_is_dropped() {
        let mixer = Arc::new(Mutex::new(Mixer::default()));
        // One other player shares the output.
        let shares = Arc::new(std::sync::atomic::AtomicUsize::new(1));
        let backend = |owner| MixerBackend {
            mixer: mixer.clone(),
            root: PathBuf::new(),
            cache: Arc::default(),
            decoder: None,
            next: (u64::from(owner)) << 40,
            live: true,
            owner,
            world: 0,
            shares: shares.clone(),
        };
        let (mut p1, mut p2) = (backend(0), backend(1));
        // P2 is listening (its camera is in the mixer), so two players share the speakers.
        mixer
            .lock()
            .unwrap()
            .listeners
            .insert(1, ([0.0; 3], 0.0, 0.0));
        let started = |b: &mut MixerBackend, at: [f64; 3]| {
            let id = b.create(false).unwrap();
            b.set_position(id, at);
            !b.duplicate(id, ("minecraft", "mob/creeper/say1.ogg"))
        };
        assert!(started(&mut p1, [5.0, 64.0, 5.0]));
        // P2's client got the same sound at the same place: one copy plays.
        assert!(!started(&mut p2, [5.2, 64.0, 5.0]));
        // Elsewhere, or P1's own second one, it is a sound of its own.
        assert!(started(&mut p2, [40.0, 64.0, 5.0]));
        assert!(started(&mut p1, [5.0, 64.0, 5.0]));
        // Equal coordinates in another dimension describe a different sound.
        p2.world = mixer.lock().unwrap().world_id("nether");
        assert!(started(&mut p2, [5.0, 64.0, 5.0]));
        // P2 leaving takes its sounds.
        drop(p2);
        assert!(
            mixer
                .lock()
                .unwrap()
                .channels
                .values()
                .all(|c| c.owner == 0)
        );
    }

    #[test]
    fn reusing_a_seat_does_not_reuse_its_channels_or_leave_its_camera() {
        let host = Audio::silent(Path::new("/nonexistent/horizon-test-audio"));
        let mut guest = host.share(1);
        guest.set_world("overworld");
        guest.set_listener([1.0, 0.0, 0.0], 0.0, 0.0);
        let old = guest.backend.create(false).unwrap();
        drop(guest);
        {
            let mixer = host.backend.mixer.lock().unwrap();
            assert!(!mixer.channels.contains_key(&old));
            assert!(!mixer.listeners.contains_key(&1));
            assert!(!mixer.worlds.contains_key(&1));
        }
        let mut replacement = host.share(1);
        let new = replacement.backend.create(false).unwrap();
        assert_ne!(old, new);
        assert!(
            replacement
                .backend
                .mixer
                .lock()
                .unwrap()
                .channels
                .contains_key(&new)
        );
    }

    #[test]
    fn a_mono_output_plays_sources_unpanned() {
        let mut m = Mixer::default();
        m.channels
            .insert(1, channel(vec![0.5; 4], None, [0.0, 0.0, -3.0], true));
        let mut data = [0.0; 2];
        mix(&mut m, &mut data, 1, 48_000);
        assert_eq!(data, [0.5; 2]);
    }
    #[cfg(feature = "audio")]
    #[test]
    fn pending_device_discovery_does_not_block_tick() {
        let mut audio = Audio::silent(Path::new("missing-audio-test-resources"));
        audio.backend.mixer.lock().unwrap().device_name = "primary".into();
        audio.default_seen = Some("primary".into());
        audio.devices_seen = vec!["primary".into()];
        let (tx, rx) = std::sync::mpsc::channel();
        audio.device_probe = Some(rx);
        assert!(!audio.should_change_device());
        assert!(audio.device_probe.is_some());
        tx.send((Some("headphones".into()), None, vec!["headphones".into()]))
            .unwrap();
        assert!(audio.should_change_device());
        assert!(audio.device_probe.is_none());
        assert_eq!(audio.default_seen.as_deref(), Some("headphones"));
    }

    #[test]
    fn stereo_files_play_as_they_are_wherever_their_source_is() {
        let mut m = Mixer::default();
        // Out of its 16-block range and off to the side: OpenAL neither pans
        // nor attenuates a stereo buffer.
        let mut c = channel(Vec::new(), Some(16.0), [-40.0, 0.0, 0.0], false);
        c.samples = Some((Arc::new([0.5f32, -0.25].repeat(8)), 48_000, 2));
        m.channels.insert(1, c);
        let mut data = [0.0f32; 8];
        mix(&mut m, &mut data, 2, 48_000);
        for frame in data.as_chunks::<2>().0 {
            assert_eq!(*frame, [0.5, -0.25]);
        }
        // A mono output hears both channels.
        m.channels.get_mut(&1).unwrap().position = 0.0;
        let mut mono = [0.0f32; 4];
        mix(&mut m, &mut mono, 1, 48_000);
        assert_eq!(mono, [0.125; 4]);
    }

    #[test]
    fn directional_audio_delays_and_dulls_the_ear_away_from_a_source() {
        let mut m = Mixer {
            directional: true,
            ..Mixer::default()
        };
        // A click west of a listener facing south: on its right.
        let mut click = vec![0.0f32; 128];
        click[0] = 1.0;
        let mut c = channel(click, None, [-4.0, 0.0, 0.0], false);
        c.gain = 0.25;
        m.channels.insert(1, c);
        let mut data = [0.0f32; 2 * 96];
        mix(&mut m, &mut data, 2, 48_000);
        let (left, right): (Vec<f32>, Vec<f32>) =
            data.as_chunks::<2>().0.iter().map(|f| (f[0], f[1])).unzip();
        // The near ear hears it at once, the far one 0.66 ms (31.5 samples)
        // later, around the head.
        assert!(right[0] > 0.0);
        assert!(left[..31].iter().all(|&s| s == 0.0), "{left:?}");
        assert_ne!(left[31], 0.0);
        let energy = |s: &[f32]| s.iter().map(|x| x * x).sum::<f32>();
        assert!(energy(&left) < energy(&right) / 2.0);
    }

    #[test]
    fn a_source_ahead_reaches_both_ears_alike_and_one_behind_loses_its_highs() {
        let ahead = ear_settings([0.0, 0.0, 3.0], 48_000);
        assert_eq!(ahead.delay[0], ahead.delay[1]);
        assert_eq!(ahead.shadow[0], ahead.shadow[1]);
        assert_eq!(ahead.back, 1.0);
        let behind = ear_settings([0.0, 0.0, -3.0], 48_000);
        assert_eq!(behind.delay, ahead.delay);
        assert!(behind.back < 0.6, "{behind:?}");
    }

    #[test]
    fn sound_physics_muffles_what_is_behind_walls_and_echoes_in_caves() {
        assert_eq!(muffle_coefficient(1.0, 48_000), 1.0);
        // Five percent of a 5 kHz tone left: a 250 Hz one-pole low-pass.
        let k = muffle_coefficient(0.05, 48_000);
        assert!((k - 0.0322).abs() < 1e-3, "{k}");
        // The highest tone a buffer can hold, behind a wall and in the open.
        let hum: Vec<f32> = (0..256)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let loudness = |physics: Option<Physics>| {
            let mut m = Mixer {
                physics: true,
                ..Mixer::default()
            };
            let mut c = channel(hum.clone(), Some(64.0), [0.0, 0.0, 4.0], false);
            c.physics = physics.map(|p| (p, std::time::Instant::now()));
            m.channels.insert(1, c);
            let mut data = [0.0f32; 2 * 200];
            mix(&mut m, &mut data, 2, 48_000);
            data[200..].iter().map(|s| s.abs()).sum::<f32>()
        };
        let open = loudness(None);
        let walled = loudness(Some(Physics {
            gain: 0.8,
            highs: 0.05,
            sends: [0.0; 4],
        }));
        assert!(walled < open * 0.1, "{walled} {open}");
        // A click in a cave rings on after it stops; in the open it doesn't.
        let ring = |sends: [f32; 4]| {
            let mut m = Mixer {
                physics: true,
                ..Mixer::default()
            };
            let mut click = vec![0.0f32; 64];
            click[0] = 1.0;
            let mut c = channel(click, Some(64.0), [0.0, 0.0, 4.0], false);
            c.physics = Some((
                Physics {
                    gain: 1.0,
                    highs: 1.0,
                    sends,
                },
                std::time::Instant::now(),
            ));
            m.channels.insert(1, c);
            let mut first = vec![0.0f32; 2 * 4800];
            mix(&mut m, &mut first, 2, 48_000);
            let mut later = vec![0.0f32; 2 * 4800];
            mix(&mut m, &mut later, 2, 48_000);
            later.iter().map(|s| s * s).sum::<f32>()
        };
        assert_eq!(ring([0.0; 4]), 0.0);
        assert!(ring([0.0, 0.0, 0.5, 0.5]) > 1e-4);
    }

    #[test]
    fn output_devices_are_looked_at_again_only_when_the_cards_change() {
        // No fingerprint (not Linux, or the cards could not be read): the list
        // has to be looked at as before.
        assert!(cards_changed(None, None));
        assert!(cards_changed(Some("0 [PCH ]"), None));
        // The same cards list the same devices; a card added or removed does not.
        assert!(!cards_changed(Some("0 [PCH ]"), Some("0 [PCH ]")));
        assert!(cards_changed(Some("0 [PCH ]"), Some("0 [PCH ]\n 1 [USB ]")));
    }
}
