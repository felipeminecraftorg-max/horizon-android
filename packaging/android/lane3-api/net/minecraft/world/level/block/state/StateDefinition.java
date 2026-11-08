// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import net.minecraft.world.level.block.state.properties.Property;

/** A block's states: lane 3's is the block's own state. */
public class StateDefinition<O, S extends StateHolder<O, S>> {
    private final S any;

    public StateDefinition(S any) {
        this.any = any;
    }

    public S any() {
        return any;
    }

    public static class Builder<O, S extends StateHolder<O, S>> {
        public Builder(O owner) {}

        public Builder<O, S> add(Property<?>... properties) {
            return this;
        }
    }
}
