// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package dev.horizon.android.lane3;

import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.function.Function;
import net.minecraft.core.DefaultedRegistry;
import net.minecraft.resources.ResourceLocation;

/**
 * A registry by id. Asking for an id the game has gives a value standing for it; what a mod
 * registers in code is kept (its content came over with lane 1).
 */
public final class SimpleRegistry<T> implements DefaultedRegistry<T> {
    private final String defaultId;
    /** The registry's own key ({@code minecraft:block}), for the entries mods register. */
    private final net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<T>> registry;
    private final Function<String, T> make;
    private final Map<ResourceLocation, T> values = new LinkedHashMap<>();
    private final Map<Object, ResourceLocation> keys = new java.util.IdentityHashMap<>();

    public SimpleRegistry(
            net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<T>> registry,
            String defaultId,
            Function<String, T> make) {
        this.registry = registry;
        this.defaultId = defaultId;
        this.make = make;
    }

    @Override
    public net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<T>> key() {
        return registry;
    }

    /** A mod's entry, made: kept as the one value for its id, without counting it again. */
    synchronized void hold(ResourceLocation id, T value) {
        values.put(id, value);
        keys.put(value, id);
    }

    /** {@link #hold} for the API slice's own classes. */
    public void horizonHold(ResourceLocation id, T value) {
        hold(id, value);
    }

    public synchronized void put(ResourceLocation id, T value) {
        values.put(id, value);
        keys.put(value, id);
        Lane3.registered("registry " + defaultId.substring(defaultId.indexOf(':') + 1));
    }

    @Override
    public synchronized ResourceLocation getKey(T value) {
        ResourceLocation key = keys.get(value);
        if (key != null) return key;
        if (value instanceof net.minecraft.world.level.block.Block b) return ResourceLocation.parse(b.id());
        if (value instanceof net.minecraft.world.item.Item i) return ResourceLocation.parse(i.id());
        return getDefaultKey();
    }

    @Override
    public synchronized T get(ResourceLocation id) {
        if (id == null) id = getDefaultKey();
        T value = values.get(id);
        if (value == null) {
            // A mod's own entry: the object it made (lane 3 makes it now if nothing asked yet).
            net.minecraft.resources.ResourceKey<T> key = net.minecraft.resources.ResourceKey.create(registry, id);
            if (Neo.has(key)) {
                @SuppressWarnings("unchecked")
                T made = (T) Neo.value(key);
                return made;
            }
            value = make.apply(id.toString());
            values.put(id, value);
            keys.put(value, id);
        }
        return value;
    }

    @Override
    public synchronized boolean containsKey(ResourceLocation id) {
        return values.containsKey(id);
    }

    @Override
    public ResourceLocation getDefaultKey() {
        return ResourceLocation.parse(defaultId);
    }

    @Override
    public synchronized Iterator<T> iterator() {
        return new java.util.ArrayList<>(values.values()).iterator();
    }
}
