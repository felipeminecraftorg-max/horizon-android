// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Optional;

/** A property whose values are an enum's: written in lower case ({@code AttachFace.WALL} is {@code wall}). */
public class EnumProperty<T extends Enum<T>> extends Property<T> {
    private final List<T> values;

    protected EnumProperty(String name, Class<T> type, Collection<T> values) {
        super(name, type);
        this.values = List.copyOf(values);
    }

    public static <T extends Enum<T>> EnumProperty<T> create(String name, Class<T> type) {
        T[] all = type.getEnumConstants();
        return new EnumProperty<>(name, type, all == null ? List.of() : List.of(all));
    }

    @SafeVarargs
    public static <T extends Enum<T>> EnumProperty<T> create(String name, Class<T> type, T... values) {
        return new EnumProperty<>(name, type, List.of(values));
    }

    public static <T extends Enum<T>> EnumProperty<T> create(String name, Class<T> type, Collection<T> values) {
        return new EnumProperty<>(name, type, values);
    }

    public static <T extends Enum<T>> EnumProperty<T> create(String name, Class<T> type, java.util.function.Predicate<T> filter) {
        List<T> kept = new ArrayList<>();
        T[] all = type.getEnumConstants();
        if (all != null) {
            for (T v : all) if (filter.test(v)) kept.add(v);
        }
        return new EnumProperty<>(name, type, kept);
    }

    @Override
    public Collection<T> getPossibleValues() {
        return values;
    }

    @Override
    public String getName(T value) {
        if (value instanceof net.minecraft.core.Direction d) return d.getName();
        return value.name().toLowerCase(java.util.Locale.ROOT);
    }

    @Override
    public Optional<T> getValue(String name) {
        for (T v : values) {
            if (getName(v).equals(name)) return Optional.of(v);
        }
        return Optional.empty();
    }

    /** The game's constants: the property and the enum its field names. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    public static EnumProperty horizonConstant(String owner, String field) {
        String f = field.toUpperCase(java.util.Locale.ROOT);
        if (f.equals("AXIS")) return create("axis", net.minecraft.core.Direction.Axis.class);
        if (f.equals("HORIZONTAL_AXIS")) {
            return create("axis", net.minecraft.core.Direction.Axis.class, net.minecraft.core.Direction.Axis.X, net.minecraft.core.Direction.Axis.Z);
        }
        String[][] known = {
            {"FACE", "face", "AttachFace"},
            {"ATTACH_FACE", "face", "AttachFace"},
            {"HALF", "half", "Half"},
            {"DOUBLE_BLOCK_HALF", "half", "DoubleBlockHalf"},
            {"SLAB_TYPE", "type", "SlabType"},
            {"STAIRS_SHAPE", "shape", "StairsShape"},
            {"DOOR_HINGE", "hinge", "DoorHingeSide"},
            {"BED_PART", "part", "BedPart"},
            {"CHEST_TYPE", "type", "ChestType"},
            {"NOTEBLOCK_INSTRUMENT", "instrument", "NoteBlockInstrument"},
            {"BELL_ATTACHMENT", "attachment", "BellAttachType"},
            {"PISTON_TYPE", "type", "PistonType"},
            {"MODE_COMPARATOR", "mode", "ComparatorMode"},
            {"STRUCTUREBLOCK_MODE", "mode", "StructureMode"},
            {"BAMBOO_LEAVES", "leaves", "BambooLeaves"},
            {"TILT", "tilt", "Tilt"},
            {"DRIPSTONE_THICKNESS", "thickness", "DripstoneThickness"},
            {"SCULK_SENSOR_PHASE", "sculk_sensor_phase", "SculkSensorPhase"},
        };
        for (String[] k : known) {
            if (!k[0].equals(f)) continue;
            try {
                Class type = Class.forName("net.minecraft.world.level.block.state.properties." + k[2], true, EnumProperty.class.getClassLoader());
                if (type.isEnum()) return create(k[1], type);
            } catch (Throwable t) {
                // The mod doesn't name that enum: no class for it.
            }
            return null;
        }
        return null;
    }
}
