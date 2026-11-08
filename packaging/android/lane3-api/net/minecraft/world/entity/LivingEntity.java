// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity;

import dev.horizon.android.lane3.Native;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.item.ItemStack;

public abstract class LivingEntity extends Entity {
    /** Whether it has the effect (an effect Horizon does not know is never had). */
    public boolean hasEffect(net.minecraft.core.Holder<net.minecraft.world.effect.MobEffect> effect) {
        return Native.hasEffect(dev.horizon.android.lane3.Neo.id(effect));
    }

    /** Gives an effect: Horizon's own (through {@code /effect}), one it does not know is not given. */
    public boolean addEffect(net.minecraft.world.effect.MobEffectInstance effect) {
        int seconds = Math.max(1, (effect.getDuration() + 19) / 20);
        return Native.command("effect give @s " + dev.horizon.android.lane3.Neo.id(effect.getEffect()) + " " + seconds + " "
                + Math.max(0, Math.min(255, effect.getAmplifier())) + " " + !effect.isVisible());
    }

    public boolean addEffect(net.minecraft.world.effect.MobEffectInstance effect, Entity source) {
        return addEffect(effect);
    }

    public boolean removeEffect(net.minecraft.core.Holder<net.minecraft.world.effect.MobEffect> effect) {
        return Native.command("effect clear @s " + dev.horizon.android.lane3.Neo.id(effect));
    }

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
