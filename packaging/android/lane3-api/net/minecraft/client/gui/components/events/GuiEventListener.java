// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components.events;

/** Something on a screen that hears the mouse and keys. */
public interface GuiEventListener {
    default boolean mouseClicked(double x, double y, int button) {
        return false;
    }

    default boolean mouseReleased(double x, double y, int button) {
        return false;
    }

    default boolean keyPressed(int key, int scan, int modifiers) {
        return false;
    }

    default boolean charTyped(char c, int modifiers) {
        return false;
    }

    default boolean isMouseOver(double x, double y) {
        return false;
    }

    default void setFocused(boolean focused) {}

    default boolean isFocused() {
        return false;
    }
}
