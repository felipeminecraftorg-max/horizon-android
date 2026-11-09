// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui;

/** The game's font, as a screen lays text out with it: about 6 pixels a letter, 9 a line. */
public class Font {
    public final int lineHeight = 9;

    public int width(String text) {
        return text == null ? 0 : text.length() * 6;
    }

    public int width(net.minecraft.network.chat.Component text) {
        return text == null ? 0 : width(text.getString());
    }
}
