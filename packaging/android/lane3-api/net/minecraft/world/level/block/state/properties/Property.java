// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

/** A block state's property ({@code waterlogged}, {@code facing}). */
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
}
