// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import net.minecraft.core.BlockPos;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.Level;

/** What a block does: a mod's block overrides what it does otherwise. */
public abstract class BlockBehaviour {
    protected final Properties properties;

    protected BlockBehaviour(Properties properties) {
        this.properties = properties == null ? Properties.of() : properties;
    }

    /** An entity is inside the block's space. */
    protected void entityInside(BlockState state, Level level, BlockPos pos, Entity entity) {}

    /** The block was put in the world in place of {@code old}. */
    protected void onPlace(BlockState state, Level level, BlockPos pos, BlockState old, boolean movedByPiston) {}

    /** A tick the block scheduled is due. */
    protected void tick(BlockState state, net.minecraft.server.level.ServerLevel level, BlockPos pos, net.minecraft.util.RandomSource random) {}

    /** Lane 3 calls a block's own {@code onPlace} through this (it is protected). */
    public static void horizonPlace(BlockBehaviour block, BlockState state, Level level, BlockPos pos, BlockState old) {
        block.onPlace(state, level, pos, old, false);
    }

    public static void horizonTick(BlockBehaviour block, BlockState state, net.minecraft.server.level.ServerLevel level, BlockPos pos,
            net.minecraft.util.RandomSource random) {
        block.tick(state, level, pos, random);
    }

    public static void horizonInside(BlockBehaviour block, BlockState state, Level level, BlockPos pos, Entity entity) {
        block.entityInside(state, level, pos, entity);
    }

    /** How a block is made: lane 3 keeps what it needs of it. */
    public static class Properties {
        public static Properties of() {
            return new Properties();
        }
    }
}
