// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui;

import net.minecraft.resources.ResourceLocation;

/**
 * What a screen draws, kept as Horizon's draw lines (tab between fields): {@code blit},
 * {@code sprite}, {@code fill}, {@code text}, {@code button}, {@code item}, {@code tooltip}.
 */
public class GuiGraphics {
    private final StringBuilder out = new StringBuilder();
    private final com.mojang.blaze3d.vertex.PoseStack pose = new com.mojang.blaze3d.vertex.PoseStack();
    private final int width;
    private final int height;

    public GuiGraphics(int width, int height) {
        this.width = width;
        this.height = height;
    }

    public com.mojang.blaze3d.vertex.PoseStack pose() {
        return pose;
    }

    public int guiWidth() {
        return width;
    }

    public int guiHeight() {
        return height;
    }

    private int x(double x) {
        return (int) Math.round(x + pose.horizonX());
    }

    private int y(double y) {
        return (int) Math.round(y + pose.horizonY());
    }

    private void line(Object... fields) {
        for (int i = 0; i < fields.length; i++) {
            if (i > 0) out.append('\t');
            out.append(String.valueOf(fields[i]).replace('\t', ' ').replace('\n', ' '));
        }
        out.append('\n');
    }

    /** What was drawn. */
    public String horizonLines() {
        return out.toString();
    }

    /** Part {@code u, v, w, h} of a {@code textureWidth} by {@code textureHeight} texture at x, y. */
    public void blit(ResourceLocation texture, int x, int y, float u, float v, int w, int h, int textureWidth, int textureHeight) {
        line("blit", texture, x(x), y(y), w, h, u, v, w, h, textureWidth, textureHeight);
    }

    public void blit(ResourceLocation texture, int x, int y, int w, int h, float u, float v, int uw, int vh, int textureWidth,
            int textureHeight) {
        line("blit", texture, x(x), y(y), w, h, u, v, uw, vh, textureWidth, textureHeight);
    }

    /** Part of a 256 by 256 texture. */
    public void blit(ResourceLocation texture, int x, int y, int u, int v, int w, int h) {
        line("blit", texture, x(x), y(y), w, h, u, v, w, h, 256, 256);
    }

    public void blit(ResourceLocation texture, int x, int y, int z, float u, float v, int w, int h, int textureWidth, int textureHeight) {
        line("blit", texture, x(x), y(y), w, h, u, v, w, h, textureWidth, textureHeight);
    }

    public void blitSprite(ResourceLocation sprite, int x, int y, int w, int h) {
        line("sprite", sprite, x(x), y(y), w, h);
    }

    public void fill(int x1, int y1, int x2, int y2, int argb) {
        line("fill", x(x1), y(y1), x(x2), y(y2), Integer.toUnsignedString(argb));
    }

    public void fill(int x1, int y1, int x2, int y2, int z, int argb) {
        fill(x1, y1, x2, y2, argb);
    }

    public void fillGradient(int x1, int y1, int x2, int y2, int from, int to) {
        fill(x1, y1, x2, y2, from);
    }

    public int drawString(Font font, String text, int x, int y, int rgb, boolean shadow) {
        if (text == null) return x;
        line("text", x(x), y(y), Integer.toUnsignedString(rgb & 0xFFFFFF), shadow ? 1 : 0, text);
        return x + (font == null ? text.length() * 6 : font.width(text));
    }

    public int drawString(Font font, String text, int x, int y, int rgb) {
        return drawString(font, text, x, y, rgb, true);
    }

    public int drawString(Font font, net.minecraft.network.chat.Component text, int x, int y, int rgb, boolean shadow) {
        return drawString(font, text == null ? null : text.getString(), x, y, rgb, shadow);
    }

    public int drawString(Font font, net.minecraft.network.chat.Component text, int x, int y, int rgb) {
        return drawString(font, text, x, y, rgb, true);
    }

    public void drawCenteredString(Font font, String text, int x, int y, int rgb) {
        if (text == null) return;
        drawString(font, text, x - (font == null ? text.length() * 3 : font.width(text) / 2), y, rgb, true);
    }

    public void drawCenteredString(Font font, net.minecraft.network.chat.Component text, int x, int y, int rgb) {
        drawCenteredString(font, text == null ? null : text.getString(), x, y, rgb);
    }

    public void renderTooltip(Font font, net.minecraft.network.chat.Component text, int x, int y) {
        if (text != null) line("tooltip", x(x), y(y), text.getString());
    }

    public void renderItem(net.minecraft.world.item.ItemStack stack, int x, int y) {
        if (stack != null && !stack.isEmpty()) line("item", x(x), y(y), stack.getItem().id(), stack.getCount());
    }

    public void renderFakeItem(net.minecraft.world.item.ItemStack stack, int x, int y) {
        renderItem(stack, x, y);
    }

    public void renderItemDecorations(Font font, net.minecraft.world.item.ItemStack stack, int x, int y) {}

    /** A vanilla button. */
    public void horizonButton(int x, int y, int w, int h, boolean hovered, boolean active, String label) {
        line("button", x(x), y(y), w, h, hovered ? 1 : 0, active ? 1 : 0, label == null ? "" : label);
    }
}
