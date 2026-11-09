// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.sounds;

/** A sound's category, as `/playsound` names it. */
public enum SoundSource {
    MASTER("master"),
    MUSIC("music"),
    RECORDS("record"),
    WEATHER("weather"),
    BLOCKS("block"),
    HOSTILE("hostile"),
    NEUTRAL("neutral"),
    PLAYERS("player"),
    AMBIENT("ambient"),
    VOICE("voice");

    private final String name;

    SoundSource(String name) {
        this.name = name;
    }

    public String getName() {
        return name;
    }
}
