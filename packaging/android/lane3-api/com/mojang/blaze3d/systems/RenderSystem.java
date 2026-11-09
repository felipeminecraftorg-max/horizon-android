// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package com.mojang.blaze3d.systems;

/** The game's draw state: lane 3's screens draw through Horizon, which sets its own. */
public final class RenderSystem {
    private RenderSystem() {}

    public static void setShaderColor(float r, float g, float b, float a) {}

    public static void enableBlend() {}

    public static void disableBlend() {}

    public static void defaultBlendFunc() {}

    public static void enableDepthTest() {}

    public static void disableDepthTest() {}

    public static void setShaderTexture(int unit, net.minecraft.resources.ResourceLocation texture) {}
}
