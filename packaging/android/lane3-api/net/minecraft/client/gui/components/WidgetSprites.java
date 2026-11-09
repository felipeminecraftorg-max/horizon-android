// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.components;

import net.minecraft.resources.ResourceLocation;

/** A widget's textures: enabled, disabled, and both while focused (hovered). */
public record WidgetSprites(ResourceLocation enabled, ResourceLocation disabled, ResourceLocation enabledFocused,
        ResourceLocation disabledFocused) {
    public WidgetSprites(ResourceLocation enabled, ResourceLocation focused) {
        this(enabled, enabled, focused, focused);
    }

    public WidgetSprites(ResourceLocation enabled, ResourceLocation disabled, ResourceLocation focused) {
        this(enabled, disabled, focused, disabled);
    }

    public ResourceLocation get(boolean isEnabled, boolean isFocused) {
        if (isEnabled) return isFocused ? enabledFocused : enabled;
        return isFocused ? disabledFocused : disabled;
    }
}
