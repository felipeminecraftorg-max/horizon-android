// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.event.entity.player;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;

/** A player used a hand on something. */
public abstract class PlayerInteractEvent extends PlayerEvent {
    private final InteractionHand hand;
    private final BlockPos pos;
    private final Direction face;
    private InteractionResult cancellationResult = InteractionResult.PASS;

    protected PlayerInteractEvent(Player player, InteractionHand hand, BlockPos pos, Direction face) {
        super(player);
        this.hand = hand;
        this.pos = pos;
        this.face = face;
    }

    public InteractionHand getHand() {
        return hand;
    }

    public ItemStack getItemStack() {
        return getEntity().getItemInHand(hand);
    }

    public BlockPos getPos() {
        return pos;
    }

    public Direction getFace() {
        return face;
    }

    public Level getLevel() {
        return getEntity().level();
    }

    public net.neoforged.fml.LogicalSide getSide() {
        return net.neoforged.fml.LogicalSide.SERVER;
    }

    public InteractionResult getCancellationResult() {
        return cancellationResult;
    }

    public void setCancellationResult(InteractionResult result) {
        this.cancellationResult = result;
    }

    /** The player right-clicked a block. */
    public static class RightClickBlock extends PlayerInteractEvent implements net.neoforged.bus.api.ICancellableEvent {
        private final net.minecraft.world.phys.BlockHitResult hit;
        private net.neoforged.neoforge.common.util.TriState useBlock = net.neoforged.neoforge.common.util.TriState.DEFAULT;
        private net.neoforged.neoforge.common.util.TriState useItem = net.neoforged.neoforge.common.util.TriState.DEFAULT;

        public RightClickBlock(Player player, InteractionHand hand, BlockPos pos, net.minecraft.world.phys.BlockHitResult hit) {
            super(player, hand, pos, hit.getDirection());
            this.hit = hit;
        }

        public net.minecraft.world.phys.BlockHitResult getHitVec() {
            return hit;
        }

        public net.neoforged.neoforge.common.util.TriState getUseBlock() {
            return useBlock;
        }

        public net.neoforged.neoforge.common.util.TriState getUseItem() {
            return useItem;
        }

        public void setUseBlock(net.neoforged.neoforge.common.util.TriState use) {
            this.useBlock = use;
        }

        public void setUseItem(net.neoforged.neoforge.common.util.TriState use) {
            this.useItem = use;
        }
    }

    /** The player right-clicked with an item, at nothing. */
    public static class RightClickItem extends PlayerInteractEvent implements net.neoforged.bus.api.ICancellableEvent {
        public RightClickItem(Player player, InteractionHand hand) {
            super(player, hand, player.blockPosition(), null);
        }
    }
}
