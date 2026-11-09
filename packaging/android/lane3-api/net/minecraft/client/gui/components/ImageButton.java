// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components;

import net.minecraft.network.chat.Component;

/** A button drawn with its own textures. */
public class ImageButton extends Button {
    protected final WidgetSprites sprites;

    public ImageButton(int x, int y, int width, int height, WidgetSprites sprites, OnPress onPress) {
        this(x, y, width, height, sprites, onPress, Component.empty());
    }

    public ImageButton(int x, int y, int width, int height, WidgetSprites sprites, OnPress onPress, Component message) {
        super(x, y, width, height, message, onPress, null);
        this.sprites = sprites;
    }

    public ImageButton(int width, int height, WidgetSprites sprites, OnPress onPress, Component message) {
        this(0, 0, width, height, sprites, onPress, message);
    }

    @Override
    public void renderWidget(net.minecraft.client.gui.GuiGraphics graphics, int mouseX, int mouseY, float partialTick) {
        graphics.blitSprite(sprites.get(isActive(), isHoveredOrFocused()), getX(), getY(), width, height);
    }
}
