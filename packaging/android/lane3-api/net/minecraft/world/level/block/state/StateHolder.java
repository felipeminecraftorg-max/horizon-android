// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import net.minecraft.world.level.block.state.properties.Property;

/** Something with property values (a block state). */
public abstract class StateHolder<O, S> {
    public abstract <T extends Comparable<T>> T getValue(Property<T> property);

    public abstract <T extends Comparable<T>> boolean hasProperty(Property<T> property);

    public abstract <T extends Comparable<T>, V extends T> S setValue(Property<T> property, V value);

    public <T extends Comparable<T>> S cycle(Property<T> property) {
        java.util.List<T> values = new java.util.ArrayList<>(property.getPossibleValues());
        int at = values.indexOf(getValue(property));
        return setValue(property, values.get((at + 1) % values.size()));
    }

    public <T extends Comparable<T>, V extends T> S trySetValue(Property<T> property, V value) {
        @SuppressWarnings("unchecked")
        S self = (S) this;
        return hasProperty(property) ? setValue(property, value) : self;
    }

    public <T extends Comparable<T>> java.util.Optional<T> getOptionalValue(Property<T> property) {
        return hasProperty(property) ? java.util.Optional.ofNullable(getValue(property)) : java.util.Optional.empty();
    }
}
