#!/usr/bin/env python3
"""Edit four downloaded Firefly WAV variations into seven quiet UI cues.

Uses ffmpeg for existing-waveform edits only; no synthesis or remote requests.
Original downloads stay outside the public source tree. Output provenance records
their hashes, the exact prompt and every edit required to reproduce the cues.
"""
import argparse
import hashlib
import json
import math
import pathlib
import struct
import subprocess

PROMPT = ('Soft futuristic console menu selection tick. Rounded low-mid glass-plastic '
          'tap with a tiny smooth airy tail. Gentle attack, warm body, dry and clean, '
          'one isolated short sound. No shrill beep, buzz, music or voices.')
# event, variation, start/end seconds, playback rate, reverse, high/low Hz, RMS/peak dBFS
RECIPES = [
    ('select', 2, .012, .220, 1.12, False, 95, 2500, -26, -14),
    ('open', 4, .075, .700, 1.25, False, 110, 2600, -27, -14),
    ('back', 4, .075, .700, 1.45, True, 110, 2100, -27, -14),
    ('confirm', 1, .035, .330, 1.15, False, 100, 2400, -25, -13),
    ('error', 3, .004, .175, .78, False, 110, 1500, -30, -17),
    ('toggle', 2, .012, .125, 1.05, False, 130, 2300, -29, -16),
    ('click', 1, .120, .188, 1.2, False, 150, 2200, -30, -17),
]


def command(*args):
    return subprocess.check_output(['ffmpeg', '-v', 'error', '-nostdin', *args])


def metrics(data):
    values = struct.unpack('<' + 'f' * (len(data) // 4), data)
    rms = math.sqrt(sum(x*x for x in values) / len(values))
    peak = max(abs(x) for x in values)
    return {'rmsDbFS': round(20*math.log10(max(rms, 1e-12)), 3),
            'peakDbFS': round(20*math.log10(max(peak, 1e-12)), 3),
            'durationSeconds': round(len(values)/48000, 6)}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-dir', type=pathlib.Path, required=True)
    parser.add_argument('--output-dir', type=pathlib.Path, required=True)
    parser.add_argument('--wav-dir', type=pathlib.Path, required=True)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    args.wav_dir.mkdir(parents=True, exist_ok=True)
    samples = []
    for event, variation, start, end, rate, reverse, high, low, rms, ceiling in RECIPES:
        source = args.source_dir / f'select-v{variation}.wav'
        filters = ['pan=mono|c0=0.5*c0+0.5*c1', f'atrim=start={start}:end={end}', 'asetpts=PTS-STARTPTS',
                   f'asetrate={48000*rate}', 'aresample=48000']
        if reverse:
            filters.append('areverse')
        duration = (end-start)/rate
        fade_out = min(.045, duration*.3)
        filters += [f'highpass=f={high}', f'lowpass=f={low}',
                    'afade=t=in:d=0.006', f'afade=t=out:st={duration-fade_out}:d={fade_out}']
        base = command('-i', str(source), '-ac', '1', '-af', ','.join(filters), '-f', 'f32le', '-')
        level = metrics(base)
        gain = min(rms-level['rmsDbFS'], ceiling-level['peakDbFS'])-.75
        wav = args.wav_dir / f'firefly-{event}.wav'
        ogg = args.output_dir / f'firefly-{event}.ogg'
        command('-y', '-i', str(source), '-ac', '1', '-af', ','.join(filters+[f'volume={gain}dB']),
                '-ar', '48000', '-c:a', 'pcm_s16le', str(wav))
        command('-y', '-i', str(wav), '-c:a', 'libvorbis', '-q:a', '5', str(ogg))
        decoded = metrics(command('-i', str(ogg), '-ac', '1', '-ar', '48000', '-f', 'f32le', '-'))
        assert decoded['peakDbFS'] < -10 and .03 < decoded['durationSeconds'] < .8, (event, decoded)
        samples.append({'event': event, 'file': 'sounds/'+ogg.name, 'sourceVariation': variation,
                        'sourceFile': source.name, 'sourceSha256': sha(source), 'sha256': sha(ogg),
                        'processing': {'trimSeconds': [start, end], 'rate': rate, 'reverse': reverse,
                                       'highpassHz': high, 'lowpassHz': low, 'fadeInMs': 6,
                                       'fadeOutMs': round(fade_out*1000, 3), 'gainDb': round(gain, 6),
                                       'mono': True, 'sampleRate': 48000, 'vorbisQuality': 5},
                        'decoded': decoded})
    manifest = {'provider': 'Adobe Firefly', 'sourcePage': 'https://firefly.adobe.com/generate/sound-effects',
                'generationDate': '2026-11-15', 'prompt': PROMPT, 'generatedDurationSeconds': 1,
                'generatedVariations': 4, 'generationLimit': 'Daily free generations exhausted after this batch.',
                'notice': 'Generated assets edited from four Firefly outputs; no CC0 grant is asserted.',
                'samples': samples}
    (args.output_dir/'sources.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps([{'event': s['event'], **s['decoded']} for s in samples], indent=2))


if __name__ == '__main__':
    main()
