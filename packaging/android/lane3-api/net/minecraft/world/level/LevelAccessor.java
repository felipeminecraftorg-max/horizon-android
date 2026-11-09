// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.state.BlockState;

/** A level's blocks, as code that may run in world generation or play sees them. */
public interface LevelAccessor {
    BlockState getBlockState(BlockPos pos);

    boolean setBlock(BlockPos pos, BlockState state, int flags);

    boolean isClientSide();

    long getDayTime();

    default net.minecraft.server.MinecraftServer getServer() {
        return net.minecraft.server.MinecraftServer.CURRENT;
    }

    /** A tick for the block at {@code pos} in {@code delay} ticks (the block's {@code tick} then runs). */
    default void scheduleTick(BlockPos pos, net.minecraft.world.level.block.Block block, int delay) {
        dev.horizon.android.lane3.Native.scheduleTick(pos.getX(), pos.getY(), pos.getZ(), delay);
    }

    /** The biome at {@code pos}, by its id. */
    default net.minecraft.core.Holder<net.minecraft.world.level.biome.Biome> getBiome(BlockPos pos) {
        String id = dev.horizon.android.lane3.Native.biome(pos.getX(), pos.getY(), pos.getZ());
        return dev.horizon.android.lane3.Neo.biome(id == null || id.isEmpty() ? "minecraft:plains" : id);
    }

    default net.minecraft.world.level.block.entity.BlockEntity getBlockEntity(BlockPos pos) {
        return null;
    }

    /** A particle only clients draw: the server shows none. */
    default void addParticle(net.minecraft.core.particles.ParticleOptions particle, double x, double y, double z, double dx, double dy,
            double dz) {}

    default net.minecraft.util.RandomSource getRandom() {
        return net.minecraft.server.level.ServerLevel.CURRENT.random;
    }
}
