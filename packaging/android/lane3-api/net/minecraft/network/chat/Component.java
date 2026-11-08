// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network.chat;

/** Text: lane 3 keeps its words. */
public interface Component {
    String getString();

    static MutableComponent literal(String text) {
        return new MutableComponent(text);
    }

    static MutableComponent translatable(String key) {
        return new MutableComponent(key);
    }

    static MutableComponent translatable(String key, Object... args) {
        return new MutableComponent(key);
    }

    static Component empty() {
        return new MutableComponent("");
    }
}
