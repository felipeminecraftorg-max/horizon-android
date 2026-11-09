// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components;

/** Something a screen draws. */
public interface Renderable {
    void render(net.minecraft.client.gui.GuiGraphics graphics, int mouseX, int mouseY, float partialTick);
}
