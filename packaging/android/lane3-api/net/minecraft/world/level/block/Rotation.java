// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block;

/** A quarter turn about the vertical axis, as structure templates and blocks take it. */
public enum Rotation {
    NONE("none"),
    CLOCKWISE_90("clockwise_90"),
    CLOCKWISE_180("180"),
    COUNTERCLOCKWISE_90("counterclockwise_90");

    private final String name;

    Rotation(String name) {
        this.name = name;
    }

    /** Its name as `/place template` reads it. */
    public String getSerializedName() {
        return name;
    }

    public Rotation getRotated(Rotation other) {
        return values()[(ordinal() + other.ordinal()) % 4];
    }

    public static Rotation getRandom(net.minecraft.util.RandomSource random) {
        return values()[random.nextInt(4)];
    }

    public net.minecraft.core.Direction rotate(net.minecraft.core.Direction direction) {
        if (direction.getAxis() == net.minecraft.core.Direction.Axis.Y) return direction;
        net.minecraft.core.Direction d = direction;
        for (int i = 0; i < ordinal(); i++) d = d.getClockWise();
        return d;
    }
}
