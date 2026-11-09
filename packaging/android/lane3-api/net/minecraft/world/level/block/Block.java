// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block;

import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.block.state.BlockState;

/** A block by its id: what it does is Horizon's (lane 1 carried a mod's own). */
public class Block extends net.minecraft.world.level.block.state.BlockBehaviour {
    String id;
    /** Its properties: the ones {@code createBlockStateDefinition} adds, then Horizon's block's. */
    protected final net.minecraft.world.level.block.state.StateDefinition<Block, BlockState> stateDefinition;
    /** Horizon's default state of this block, once asked. */
    private BlockState defaultState;

    /** The game's default state of a block is Horizon's: a mod's {@code registerDefaultState} is kept by it. */
    protected void registerDefaultState(BlockState state) {}

    /** The block adds its properties ({@code builder.add(FACING)}). */
    protected void createBlockStateDefinition(
            net.minecraft.world.level.block.state.StateDefinition.Builder<Block, BlockState> builder) {}

    public net.minecraft.world.level.block.state.StateDefinition<Block, BlockState> getStateDefinition() {
        return stateDefinition;
    }

    private net.minecraft.world.level.block.state.StateDefinition<Block, BlockState> define() {
        net.minecraft.world.level.block.state.StateDefinition.Builder<Block, BlockState> builder =
                new net.minecraft.world.level.block.state.StateDefinition.Builder<>(this);
        try {
            createBlockStateDefinition(builder);
        } catch (RuntimeException | LinkageError e) {
            // A property a stand-in couldn't make: the block keeps the others.
        }
        return new net.minecraft.world.level.block.state.StateDefinition<>(this, BlockState.horizonOf(this), builder.horizonProperties());
    }

    /** A stand-in's block (one of the game's kinds the mod extends): made as a mod's is. */
    protected Block() {
        this((net.minecraft.world.level.block.state.BlockBehaviour.Properties) null);
    }

    public Block(String id) {
        super(null);
        this.id = id;
        this.stateDefinition = define();
    }

    /** A mod's block: its id is the one it is registered under (lane 3 sets it). */
    public Block(net.minecraft.world.level.block.state.BlockBehaviour.Properties properties) {
        super(properties);
        this.stateDefinition = define();
    }

    /**
     * NeoForge's: a player broke the block (the game has taken it away already: lane 3 calls
     * this after).
     */
    public boolean onDestroyedByPlayer(BlockState state, net.minecraft.world.level.Level level, net.minecraft.core.BlockPos pos,
            net.minecraft.world.entity.player.Player player, boolean willHarvest, net.minecraft.world.level.material.FluidState fluid) {
        return true;
    }

    /** The id lane 3 registered this block under. */
    public final void horizonId(String id) {
        if (this.id == null) this.id = id;
    }

    public BlockState defaultBlockState() {
        BlockState d = horizonDefault();
        return d != null ? d : BlockState.horizonOf(this);
    }

    /** Horizon's default state of the block (every property set), or null outside a game. */
    public BlockState horizonDefault() {
        if (defaultState != null || id == null) return defaultState;
        String text;
        try {
            text = dev.horizon.android.lane3.Native.defaultState(id);
        } catch (Throwable t) {
            text = null;
        }
        if (text != null && !text.isEmpty()) defaultState = BlockState.of(text);
        return defaultState;
    }

    /** A constant of the game's ({@code Blocks.STONE}): the block of the id its field names. */
    public static Block horizonConstant(String owner, String field) {
        if (!owner.endsWith("/Blocks")) return null;
        String id = "minecraft:" + field.toLowerCase(java.util.Locale.ROOT);
        // The flower pot mods cast to its kind to add their plants to it.
        if (field.equals("FLOWER_POT")) {
            try {
                Object pot = Class.forName("net.minecraft.world.level.block.FlowerPotBlock", true, Block.class.getClassLoader())
                        .getDeclaredConstructor().newInstance();
                if (pot instanceof Block b) {
                    b.horizonId(id);
                    ((dev.horizon.android.lane3.SimpleRegistry<Block>) BuiltInRegistries.BLOCK).horizonHold(net.minecraft.resources.ResourceLocation.parse(id), b);
                    return b;
                }
            } catch (Throwable t) {
                // No mod names the pot's kind: the plain block.
            }
        }
        return byId(id);
    }

    public String getDescriptionId() {
        int colon = id.indexOf(':');
        return "block." + id.substring(0, colon) + "." + id.substring(colon + 1);
    }

    public String id() {
        return id;
    }

    public static Block byId(String id) {
        return BuiltInRegistries.BLOCK.get(net.minecraft.resources.ResourceLocation.parse(id));
    }

    @Override
    public boolean equals(Object o) {
        return o == this || (o instanceof Block b && id != null && id.equals(b.id));
    }

    @Override
    public int hashCode() {
        return id == null ? System.identityHashCode(this) : id.hashCode();
    }

    @Override
    public String toString() {
        return "Block{" + id + "}";
    }
}
