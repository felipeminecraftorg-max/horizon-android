// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.registries;

import java.util.Optional;
import java.util.function.Supplier;
import net.minecraft.core.Holder;
import net.minecraft.core.Registry;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

/** A registry entry a mod registered: its value is made the first time it is asked for. */
public class DeferredHolder<R, T extends R> implements Holder<R>, Supplier<T> {
    protected final ResourceKey<R> key;

    protected DeferredHolder(ResourceKey<R> key) {
        this.key = key;
    }

    public static <R, T extends R> DeferredHolder<R, T> create(ResourceKey<R> key) {
        return new DeferredHolder<>(key);
    }

    public static <R, T extends R> DeferredHolder<R, T> create(ResourceKey<? extends Registry<R>> registry, ResourceLocation id) {
        return create(ResourceKey.create(registry, id));
    }

    public static <R, T extends R> DeferredHolder<R, T> create(ResourceLocation registry, ResourceLocation id) {
        return create(ResourceKey.create(ResourceKey.<R>createRegistryKey(registry), id));
    }

    @Override
    @SuppressWarnings("unchecked")
    public T get() {
        return (T) dev.horizon.android.lane3.Neo.value(key);
    }

    @Override
    public R value() {
        return get();
    }

    public ResourceLocation getId() {
        return key.location();
    }

    public ResourceKey<R> getKey() {
        return key;
    }

    @Override
    public boolean isBound() {
        return dev.horizon.android.lane3.Neo.has(key);
    }

    @Override
    public boolean is(ResourceLocation id) {
        return key.location().equals(id);
    }

    @Override
    public boolean is(ResourceKey<R> other) {
        return key.equals(other);
    }

    @Override
    public Optional<ResourceKey<R>> unwrapKey() {
        return Optional.of(key);
    }

    @Override
    public String getRegisteredName() {
        return key.location().toString();
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof DeferredHolder<?, ?> h && h.key.equals(key);
    }

    @Override
    public int hashCode() {
        return key.hashCode();
    }

    @Override
    public String toString() {
        return "DeferredHolder{" + key + "}";
    }
}
