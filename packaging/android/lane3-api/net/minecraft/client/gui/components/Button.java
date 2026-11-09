// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components;

import net.minecraft.network.chat.Component;

/** The game's button: what it does when pressed. */
public class Button extends AbstractButton {
    protected final OnPress onPress;

    protected Button(int x, int y, int width, int height, Component message, OnPress onPress, CreateNarration narration) {
        super(x, y, width, height, message);
        this.onPress = onPress;
    }

    public static Builder builder(Component message, OnPress onPress) {
        return new Builder(message, onPress);
    }

    @Override
    public void onPress() {
        if (onPress != null) onPress.onPress(this);
    }

    @FunctionalInterface
    public interface OnPress {
        void onPress(Button button);
    }

    @FunctionalInterface
    public interface CreateNarration {
        Object createNarrationMessage(java.util.function.Supplier<Object> defaultNarration);
    }

    public static class Builder {
        private final Component message;
        private final OnPress onPress;
        private int x;
        private int y;
        private int width = 150;
        private int height = 20;

        public Builder(Component message, OnPress onPress) {
            this.message = message;
            this.onPress = onPress;
        }

        public Builder pos(int x, int y) {
            this.x = x;
            this.y = y;
            return this;
        }

        public Builder width(int width) {
            this.width = width;
            return this;
        }

        public Builder size(int width, int height) {
            this.width = width;
            this.height = height;
            return this;
        }

        public Builder bounds(int x, int y, int width, int height) {
            return pos(x, y).size(width, height);
        }

        public Builder tooltip(Object tooltip) {
            return this;
        }

        public Button build() {
            return new Button(x, y, width, height, message, onPress, null);
        }
    }
}
