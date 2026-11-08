// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity;

import dev.horizon.android.lane3.Native;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;

public abstract class LivingEntity extends Entity {
    public ItemStack getItemInHand(InteractionHand hand) {
        return ItemStack.horizon(Native.held(hand.ordinal()));
    }

    public ItemStack getMainHandItem() {
        return getItemInHand(InteractionHand.MAIN_HAND);
    }

    public ItemStack getOffhandItem() {
        return getItemInHand(InteractionHand.OFF_HAND);
    }

    public void swing(InteractionHand hand) {
        Native.swing(hand.ordinal());
    }

    public void swing(InteractionHand hand, boolean updateSelf) {
        Native.swing(hand.ordinal());
    }
}
