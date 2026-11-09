// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level;

import dev.horizon.android.lane3.Native;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.state.BlockState;

/** The level the server thread is ticking, through Horizon. */
public abstract class Level implements LevelAccessor {
    /** The vanilla levels' keys: a mod compares them with {@code ==}, as ResourceKeys are one per id. */
    public static final net.minecraft.resources.ResourceKey<Level> OVERWORLD = key("minecraft:overworld");
    public static final net.minecraft.resources.ResourceKey<Level> NETHER = key("minecraft:the_nether");
    public static final net.minecraft.resources.ResourceKey<Level> END = key("minecraft:the_end");

    @SuppressWarnings("unchecked")
    private static net.minecraft.resources.ResourceKey<Level> key(String id) {
        return (net.minecraft.resources.ResourceKey<Level>) (net.minecraft.resources.ResourceKey<?>) net.minecraft.resources.ResourceKey.create(
                net.minecraft.core.registries.Registries.DIMENSION, net.minecraft.resources.ResourceLocation.parse(id));
    }

    /** The level the acting player is in. */
    public net.minecraft.resources.ResourceKey<Level> dimension() {
        String[] p = dev.horizon.android.lane3.Neo.player();
        return key(p.length > 5 ? p[5] : "minecraft:overworld");
    }

    public java.util.List<net.minecraft.world.entity.player.Player> players() {
        return java.util.Collections.singletonList(net.minecraft.server.level.ServerPlayer.ACTING);
    }

    @Override
    public net.minecraft.server.MinecraftServer getServer() {
        return net.minecraft.server.MinecraftServer.CURRENT;
    }

    public final boolean isClientSide = false;

    public final net.minecraft.util.RandomSource random = net.minecraft.util.RandomSource.create();

    @Override
    public net.minecraft.util.RandomSource getRandom() {
        return random;
    }

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

    public net.minecraft.world.level.material.FluidState getFluidState(BlockPos pos) {
        return new net.minecraft.world.level.material.FluidState();
    }

    public boolean isEmptyBlock(BlockPos pos) {
        return getBlockState(pos).isAir();
    }
}
