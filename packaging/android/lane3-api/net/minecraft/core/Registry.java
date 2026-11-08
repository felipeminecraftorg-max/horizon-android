// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

import net.minecraft.resources.ResourceLocation;

/** A registry: lane 3 records what a mod registers (its data came over with lane 1). */
public interface Registry<T> extends Iterable<T> {
    ResourceLocation getKey(T value);

    T get(ResourceLocation id);

    boolean containsKey(ResourceLocation id);

    static <V, T extends V> T register(Registry<V> registry, ResourceLocation id, T value) {
        ((dev.horizon.android.lane3.SimpleRegistry<V>) registry).put(id, value);
        return value;
    }

    static <V, T extends V> T register(Registry<V> registry, String id, T value) {
        return register(registry, ResourceLocation.parse(id), value);
    }
}
