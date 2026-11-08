// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import net.minecraft.world.level.block.state.properties.Property;

/** Something with property values: lane 3's block states keep Horizon's text of them. */
public abstract class StateHolder<O, S> {
    /** A property's value: lane 3 answers no value a mod set (false for a flag). */
    @SuppressWarnings({"unchecked", "rawtypes"})
    public <T extends Comparable<T>> T getValue(Property<T> property) {
        return property != null && property.getValueClass() == Boolean.class ? (T) Boolean.FALSE : null;
    }

    public <T extends Comparable<T>> boolean hasProperty(Property<T> property) {
        return false;
    }

    /** The state with a property set: lane 3's states are the block's own. */
    @SuppressWarnings("unchecked")
    public <T extends Comparable<T>, V extends T> S setValue(Property<T> property, V value) {
        return (S) this;
    }

    public <T extends Comparable<T>> S cycle(Property<T> property) {
        @SuppressWarnings("unchecked")
        S self = (S) this;
        return self;
    }
}
