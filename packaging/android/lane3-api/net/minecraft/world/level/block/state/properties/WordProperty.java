// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

import java.util.Collection;
import java.util.List;
import java.util.Optional;

/** A property of Horizon's block known only by its words (an enum no mod names). */
public final class WordProperty extends Property<String> {
    private final List<String> values;

    public WordProperty(String name, Collection<String> values) {
        super(name, String.class);
        this.values = List.copyOf(values);
    }

    @Override
    public Collection<String> getPossibleValues() {
        return values;
    }

    @Override
    public String getName(String value) {
        return value;
    }

    @Override
    public Optional<String> getValue(String name) {
        return values.contains(name) ? Optional.of(name) : Optional.empty();
    }
}
