// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

import java.util.Collection;
import java.util.List;
import net.minecraft.core.Direction;

/** A property that is a direction ({@code facing}). */
public final class DirectionProperty extends EnumProperty<Direction> {
    private static final List<Direction> HORIZONTAL = List.of(Direction.NORTH, Direction.EAST, Direction.SOUTH, Direction.WEST);

    private DirectionProperty(String name, Collection<Direction> values) {
        super(name, Direction.class, values);
    }

    public static DirectionProperty create(String name) {
        return new DirectionProperty(name, List.of(Direction.values()));
    }

    public static DirectionProperty create(String name, Direction... values) {
        return new DirectionProperty(name, List.of(values));
    }

    public static DirectionProperty create(String name, Collection<Direction> values) {
        return new DirectionProperty(name, values);
    }

    public static DirectionProperty create(String name, java.util.function.Predicate<Direction> filter) {
        return new DirectionProperty(name, java.util.Arrays.stream(Direction.values()).filter(filter).toList());
    }

    /**
     * The game's constants: {@code HorizontalDirectionalBlock.FACING} and
     * {@code BlockStateProperties.HORIZONTAL_FACING} face the four sides, {@code FACING} all six.
     */
    public static DirectionProperty horizonConstant(String owner, String field) {
        String f = field.toUpperCase(java.util.Locale.ROOT);
        switch (f) {
            case "HORIZONTAL_FACING":
                return new DirectionProperty("facing", HORIZONTAL);
            case "FACING_HOPPER":
                return create("facing", Direction.DOWN, Direction.NORTH, Direction.SOUTH, Direction.WEST, Direction.EAST);
            case "VERTICAL_DIRECTION":
                return create("vertical_direction", Direction.UP, Direction.DOWN);
            case "FACING":
                if (owner.endsWith("/HorizontalDirectionalBlock") || owner.endsWith("/FaceAttachedHorizontalDirectionalBlock")
                        || owner.endsWith("/AbstractFurnaceBlock") || owner.endsWith("/ChestBlock") || owner.endsWith("/DoorBlock")
                        || owner.endsWith("/StairBlock") || owner.endsWith("/TrapDoorBlock") || owner.endsWith("/BedBlock")) {
                    return new DirectionProperty("facing", HORIZONTAL);
                }
                return create("facing");
            default:
                return create(f.toLowerCase(java.util.Locale.ROOT));
        }
    }
}
