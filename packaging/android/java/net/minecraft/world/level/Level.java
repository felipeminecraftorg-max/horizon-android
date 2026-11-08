// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level;

import dev.horizon.android.lane3.Native;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.state.BlockState;

/** The level the server thread is ticking, through Horizon. */
public abstract class Level {
    public final boolean isClientSide = false;

    public BlockState getBlockState(BlockPos pos) {
        return BlockState.of(Native.block(pos.getX(), pos.getY(), pos.getZ()));
    }

    public boolean setBlock(BlockPos pos, BlockState state, int flags) {
        return Native.setBlock(pos.getX(), pos.getY(), pos.getZ(), state.horizonState(), flags);
    }

    public boolean setBlockAndUpdate(BlockPos pos, BlockState state) {
        return setBlock(pos, state, 3);
    }

    public boolean removeBlock(BlockPos pos, boolean moving) {
        return setBlock(pos, BlockState.of("minecraft:air"), 3);
    }

    public boolean destroyBlock(BlockPos pos, boolean drop) {
        return Native.destroy(pos.getX(), pos.getY(), pos.getZ(), drop);
    }

    public boolean isClientSide() {
        return false;
    }

    public long getGameTime() {
        return Native.time();
    }

    public long getDayTime() {
        return Native.time();
    }

    public boolean isEmptyBlock(BlockPos pos) {
        return getBlockState(pos).isAir();
    }
}
