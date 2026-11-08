// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block;

/** A reflection, as structure templates take it. */
public enum Mirror {
    NONE("none"),
    LEFT_RIGHT("left_right"),
    FRONT_BACK("front_back");

    private final String name;

    Mirror(String name) {
        this.name = name;
    }

    public String getSerializedName() {
        return name;
    }

    public Rotation getRotation(net.minecraft.core.Direction direction) {
        net.minecraft.core.Direction.Axis axis = direction.getAxis();
        boolean turns = (this == LEFT_RIGHT && axis == net.minecraft.core.Direction.Axis.Z)
                || (this == FRONT_BACK && axis == net.minecraft.core.Direction.Axis.X);
        return turns ? Rotation.CLOCKWISE_180 : Rotation.NONE;
    }
}
