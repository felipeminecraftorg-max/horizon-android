// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

import java.util.Collection;
import java.util.List;
import java.util.Optional;

/** A property that is true or false. */
public final class BooleanProperty extends Property<Boolean> {
    private static final List<Boolean> VALUES = List.of(true, false);

    private BooleanProperty(String name) {
        super(name, Boolean.class);
    }

    public static BooleanProperty create(String name) {
        return new BooleanProperty(name);
    }

    @Override
    public Collection<Boolean> getPossibleValues() {
        return VALUES;
    }

    @Override
    public String getName(Boolean value) {
        return value.toString();
    }

    @Override
    public Optional<Boolean> getValue(String name) {
        return name.equals("true") ? Optional.of(true) : name.equals("false") ? Optional.of(false) : Optional.empty();
    }

    /** A constant of the game's ({@code BlockStateProperties.WATERLOGGED}): named as its field is. */
    public static BooleanProperty horizonConstant(String owner, String field) {
        return create(field.toLowerCase(java.util.Locale.ROOT));
    }
}
