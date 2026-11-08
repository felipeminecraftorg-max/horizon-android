// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

import net.minecraft.resources.ResourceLocation;

public interface DefaultedRegistry<T> extends Registry<T> {
    ResourceLocation getDefaultKey();
}
