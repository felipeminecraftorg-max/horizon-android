// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network.chat;

/** Text that more can be added to. */
public class MutableComponent implements Component {
    private final StringBuilder text;

    MutableComponent(String text) {
        this.text = new StringBuilder(text);
    }

    @Override
    public String getString() {
        return text.toString();
    }

    public MutableComponent append(String more) {
        text.append(more);
        return this;
    }

    public MutableComponent append(Component more) {
        text.append(more.getString());
        return this;
    }

    @Override
    public String toString() {
        return getString();
    }
}
