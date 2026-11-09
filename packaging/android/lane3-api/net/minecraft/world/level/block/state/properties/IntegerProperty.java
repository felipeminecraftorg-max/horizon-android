// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Optional;

/** A property that is a whole number from a minimum to a maximum. */
public final class IntegerProperty extends Property<Integer> {
    private final List<Integer> values;
    private final int min;
    private final int max;

    private IntegerProperty(String name, int min, int max) {
        super(name, Integer.class);
        this.min = min;
        this.max = max;
        List<Integer> v = new ArrayList<>();
        for (int i = min; i <= max; i++) v.add(i);
        this.values = List.copyOf(v);
    }

    public static IntegerProperty create(String name, int min, int max) {
        return new IntegerProperty(name, min, max);
    }

    @Override
    public Collection<Integer> getPossibleValues() {
        return values;
    }

    @Override
    public String getName(Integer value) {
        return value.toString();
    }

    @Override
    public Optional<Integer> getValue(String name) {
        try {
            int v = Integer.parseInt(name);
            return v >= min && v <= max ? Optional.of(v) : Optional.empty();
        } catch (NumberFormatException e) {
            return Optional.empty();
        }
    }

    /** The game's constants: {@code AGE_7} is {@code age} from 0 to 7, {@code LAYERS} 1 to 8. */
    public static IntegerProperty horizonConstant(String owner, String field) {
        String f = field.toUpperCase(java.util.Locale.ROOT);
        switch (f) {
            case "LAYERS": return create("layers", 1, 8);
            case "LEVEL": return create("level", 0, 15);
            case "LEVEL_CAULDRON": return create("level", 1, 3);
            case "LEVEL_COMPOSTER": return create("level", 0, 8);
            case "LEVEL_FLOWING": return create("level", 1, 8);
            case "LEVEL_HONEY": return create("honey_level", 0, 5);
            case "POWER": return create("power", 0, 15);
            case "MOISTURE": return create("moisture", 0, 7);
            case "DISTANCE": return create("distance", 1, 7);
            case "STABILITY_DISTANCE": return create("distance", 0, 7);
            case "NOTE": return create("note", 0, 24);
            case "BITES": return create("bites", 0, 6);
            case "DELAY": return create("delay", 1, 4);
            case "EGGS": return create("eggs", 1, 4);
            case "HATCH": return create("hatch", 0, 2);
            case "PICKLES": return create("pickles", 1, 4);
            case "CANDLES": return create("candles", 1, 4);
            case "STAGE": return create("stage", 0, 1);
            case "DUSTED": return create("dusted", 0, 3);
            case "RESPAWN_ANCHOR_CHARGES": return create("charges", 0, 4);
            case "ROTATION_16": return create("rotation", 0, 15);
            case "FLOWER_AMOUNT": return create("flower_amount", 1, 4);
            case "SEGMENT_AMOUNT": return create("segment_amount", 1, 4);
            default:
                break;
        }
        int under = f.lastIndexOf('_');
        if (under > 0) {
            try {
                return create(f.substring(0, under).toLowerCase(java.util.Locale.ROOT), 0, Integer.parseInt(f.substring(under + 1)));
            } catch (NumberFormatException e) {
                // Not NAME_N.
            }
        }
        return create(f.toLowerCase(java.util.Locale.ROOT), 0, 15);
    }
}
