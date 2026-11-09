// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

import java.util.Collection;
import java.util.Optional;

/** A block state's property ({@code waterlogged}, {@code facing}): its values as Horizon writes them. */
public abstract class Property<T extends Comparable<T>> {
    private final String name;
    private final Class<T> type;

    protected Property(String name, Class<T> type) {
        this.name = name;
        this.type = type;
    }

    public String getName() {
        return name;
    }

    public Class<T> getValueClass() {
        return type;
    }

    public abstract Collection<T> getPossibleValues();

    /** A value as a state's text names it ({@code true}, {@code 7}, {@code north}). */
    public abstract String getName(T value);

    public abstract Optional<T> getValue(String name);

    /** A value of this property's from its text, or null. */
    public T horizonValue(String text) {
        return text == null ? null : getValue(text).orElse(null);
    }

    /** {@code value}'s text, for a value given untyped. */
    @SuppressWarnings("unchecked")
    public String horizonName(Object value) {
        if (value == null) return null;
        try {
            return getName((T) value);
        } catch (ClassCastException e) {
            return String.valueOf(value).toLowerCase(java.util.Locale.ROOT);
        }
    }

    /**
     * The property Horizon's block has as {@code name} with the values {@code values} (in the
     * game's order): a flag, a number, a direction or an axis, else one by its words.
     */
    public static Property<?> horizon(String name, java.util.List<String> values) {
        if (!values.isEmpty() && values.stream().allMatch(v -> v.equals("true") || v.equals("false"))) {
            return BooleanProperty.create(name);
        }
        if (!values.isEmpty() && values.stream().allMatch(v -> v.matches("-?\\d+"))) {
            int min = values.stream().mapToInt(Integer::parseInt).min().getAsInt();
            int max = values.stream().mapToInt(Integer::parseInt).max().getAsInt();
            return IntegerProperty.create(name, min, max);
        }
        java.util.List<net.minecraft.core.Direction> directions = new java.util.ArrayList<>();
        for (String v : values) {
            for (net.minecraft.core.Direction d : net.minecraft.core.Direction.values()) {
                if (d.getName().equals(v)) directions.add(d);
            }
        }
        if (!values.isEmpty() && directions.size() == values.size()) {
            return DirectionProperty.create(name, directions);
        }
        java.util.List<net.minecraft.core.Direction.Axis> axes = new java.util.ArrayList<>();
        for (String v : values) {
            for (net.minecraft.core.Direction.Axis a : net.minecraft.core.Direction.Axis.values()) {
                if (a.getName().equals(v)) axes.add(a);
            }
        }
        if (!values.isEmpty() && axes.size() == values.size()) {
            return EnumProperty.create(name, net.minecraft.core.Direction.Axis.class, axes);
        }
        return new WordProperty(name, values);
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Property<?> p && p.name.equals(name) && java.util.Objects.equals(p.type, type);
    }

    @Override
    public int hashCode() {
        return 31 * name.hashCode() + (type == null ? 0 : type.hashCode());
    }

    @Override
    public String toString() {
        return getClass().getSimpleName() + "{name=" + name + "}";
    }
}
