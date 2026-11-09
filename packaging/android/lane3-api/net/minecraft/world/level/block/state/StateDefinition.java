// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import net.minecraft.world.level.block.state.properties.Property;

/**
 * A block's properties: the ones its mod defines ({@code createBlockStateDefinition}), then the
 * ones Horizon's block of that id has.
 */
public class StateDefinition<O, S extends StateHolder<O, S>> {
    private final O owner;
    private final S any;
    private final Map<String, Property<?>> own = new LinkedHashMap<>();
    private Map<String, Property<?>> all;

    public StateDefinition(O owner, S any, Collection<Property<?>> properties) {
        this.owner = owner;
        this.any = any;
        for (Property<?> p : properties) own.put(p.getName(), p);
    }

    public S any() {
        return any;
    }

    public O getOwner() {
        return owner;
    }

    public Property<?> getProperty(String name) {
        return properties().get(name);
    }

    public Collection<Property<?>> getProperties() {
        return properties().values();
    }

    public List<S> getPossibleStates() {
        return List.of(any);
    }

    private synchronized Map<String, Property<?>> properties() {
        if (all != null) return all;
        Map<String, Property<?>> merged = new LinkedHashMap<>(own);
        String id = owner instanceof net.minecraft.world.level.block.Block b ? b.id() : null;
        if (id == null) return merged;
        String text;
        try {
            text = dev.horizon.android.lane3.Native.properties(id);
        } catch (Throwable t) {
            text = null;
        }
        // Outside a game Horizon answers nothing: asked again next time.
        if (text == null || text.isEmpty()) return merged;
        for (String part : text.split(";")) {
            int eq = part.indexOf('=');
            if (eq <= 0) continue;
            String name = part.substring(0, eq);
            if (!merged.containsKey(name)) merged.put(name, Property.horizon(name, List.of(part.substring(eq + 1).split(","))));
        }
        all = merged;
        return merged;
    }

    public static class Builder<O, S extends StateHolder<O, S>> {
        private final O owner;
        private final Map<String, Property<?>> properties = new LinkedHashMap<>();

        public Builder(O owner) {
            this.owner = owner;
        }

        public Builder<O, S> add(Property<?>... add) {
            for (Property<?> p : add) {
                if (p != null) properties.put(p.getName(), p);
            }
            return this;
        }

        /** What the block added. */
        public Collection<Property<?>> horizonProperties() {
            return properties.values();
        }
    }
}
