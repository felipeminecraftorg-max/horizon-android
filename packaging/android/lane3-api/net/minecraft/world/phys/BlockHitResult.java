// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.phys;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;

public class BlockHitResult extends HitResult {
    private final BlockPos pos;
    private final Direction direction;
    private final boolean inside;

    /** A hit on the middle of the block's face {@code direction} (Horizon doesn't pass the point). */
    public BlockHitResult(BlockPos pos, Direction direction, boolean inside) {
        this(new Vec3(pos.getX() + 0.5 + direction.getStepX() * 0.5, pos.getY() + 0.5 + direction.getStepY() * 0.5,
                pos.getZ() + 0.5 + direction.getStepZ() * 0.5), direction, pos, inside);
    }

    public BlockHitResult(Vec3 location, Direction direction, BlockPos pos, boolean inside) {
        super(location);
        this.pos = pos;
        this.direction = direction;
        this.inside = inside;
    }

    public BlockPos getBlockPos() {
        return pos;
    }

    public Direction getDirection() {
        return direction;
    }

    public boolean isInside() {
        return inside;
    }

    @Override
    public Type getType() {
        return Type.BLOCK;
    }
}
