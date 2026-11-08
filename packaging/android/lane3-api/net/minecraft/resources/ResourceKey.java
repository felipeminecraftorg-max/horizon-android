// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.resources;

import java.util.Objects;
import java.util.Optional;
import net.minecraft.core.Registry;

/** An id in a registry: the registry's name and the value's. */
public final class ResourceKey<T> implements Comparable<ResourceKey<?>> {
    private final ResourceLocation registryName;
    private final ResourceLocation location;

    private ResourceKey(ResourceLocation registryName, ResourceLocation location) {
        this.registryName = registryName;
        this.location = location;
    }

    /** One key a registry and id, as the game keeps them: a mod may compare keys with {@code ==}. */
    private static final java.util.Map<ResourceKey<?>, ResourceKey<?>> KEYS = new java.util.concurrent.ConcurrentHashMap<>();

    @SuppressWarnings("unchecked")
    private static <T> ResourceKey<T> intern(ResourceLocation registry, ResourceLocation location) {
        ResourceKey<T> made = new ResourceKey<>(registry, location);
        return (ResourceKey<T>) KEYS.computeIfAbsent(made, k -> k);
    }

    public static <T> ResourceKey<T> create(ResourceKey<? extends Registry<T>> registry, ResourceLocation location) {
        return intern(registry.location, location);
    }

    public static <T> ResourceKey<Registry<T>> createRegistryKey(ResourceLocation location) {
        return intern(ResourceLocation.withDefaultNamespace("root"), location);
    }

    public ResourceLocation location() {
        return location;
    }

    public ResourceLocation registry() {
        return registryName;
    }

    public boolean isFor(ResourceKey<? extends Registry<?>> registry) {
        return registryName.equals(registry.location());
    }

    @SuppressWarnings("unchecked")
    public <E> Optional<ResourceKey<E>> cast(ResourceKey<? extends Registry<E>> registry) {
        return isFor(registry) ? Optional.of((ResourceKey<E>) this) : Optional.empty();
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof ResourceKey<?> k && k.registryName.equals(registryName) && k.location.equals(location);
    }

    @Override
    public int hashCode() {
        return Objects.hash(registryName, location);
    }

    @Override
    public int compareTo(ResourceKey<?> o) {
        int c = registryName.compareTo(o.registryName);
        return c != 0 ? c : location.compareTo(o.location);
    }

    @Override
    public String toString() {
        return "ResourceKey[" + registryName + " / " + location + "]";
    }
}
