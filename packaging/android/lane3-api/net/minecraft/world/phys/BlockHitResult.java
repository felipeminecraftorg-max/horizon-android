// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.phys;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;

public class BlockHitResult extends HitResult {
    private final BlockPos pos;
    private final Direction direction;
    private final boolean inside;

    public BlockHitResult(BlockPos pos, Direction direction, boolean inside) {
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
