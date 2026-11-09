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

    public final boolean isClientSide;
    private final net.minecraft.world.level.chunk.ChunkSource chunks;

    protected Level() {
        this(false);
    }

    /** A level of the client's side ({@code client}): its chunks are none the server's code reaches. */
    protected Level(boolean client) {
        this.isClientSide = client;
        this.chunks = client ? new net.minecraft.world.level.chunk.ChunkSource(false) : new net.minecraft.server.level.ServerChunkCache();
    }

    public net.minecraft.world.level.chunk.ChunkSource getChunkSource() {
        return chunks;
    }

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
        return isClientSide;
    }

    public long getGameTime() {
        return Native.time();
    }

    public long getDayTime() {
        return Native.time();
    }

    /** A sound at the middle of {@code pos} for the players near it (Horizon's {@code /playsound}). */
    public void playSound(net.minecraft.world.entity.player.Player except, BlockPos pos, net.minecraft.sounds.SoundEvent sound,
            net.minecraft.sounds.SoundSource source, float volume, float pitch) {
        playSound(except, pos.getX() + 0.5, pos.getY() + 0.5, pos.getZ() + 0.5, sound, source, volume, pitch);
    }

    public void playSound(net.minecraft.world.entity.player.Player except, double x, double y, double z,
            net.minecraft.sounds.SoundEvent sound, net.minecraft.sounds.SoundSource source, float volume, float pitch) {
        if (sound == null || sound.getLocation() == null) return;
        Native.command(String.format(java.util.Locale.ROOT, "playsound %s %s @a %.3f %.3f %.3f %.3f %.3f", sound.getLocation(),
                source == null ? "master" : source.getName(), x, y, z, Math.max(0.0F, volume), Math.max(0.0F, Math.min(2.0F, pitch))));
    }

    public void playSound(net.minecraft.world.entity.player.Player except, net.minecraft.world.entity.Entity at,
            net.minecraft.sounds.SoundEvent sound, net.minecraft.sounds.SoundSource source, float volume, float pitch) {
        if (at != null) playSound(except, at.getX(), at.getY(), at.getZ(), sound, source, volume, pitch);
    }

    /** A sound only this client hears: the server has none. */
    public void playLocalSound(double x, double y, double z, net.minecraft.sounds.SoundEvent sound, net.minecraft.sounds.SoundSource source,
            float volume, float pitch, boolean delayed) {}

    /** A block's entity: lane 3 has none of its own yet (Horizon keeps the blocks' data). */
    public net.minecraft.world.level.block.entity.BlockEntity getBlockEntity(BlockPos pos) {
        return null;
    }

    public net.minecraft.world.level.material.FluidState getFluidState(BlockPos pos) {
        return new net.minecraft.world.level.material.FluidState();
    }

    public boolean isEmptyBlock(BlockPos pos) {
        return getBlockState(pos).isAir();
    }
}
