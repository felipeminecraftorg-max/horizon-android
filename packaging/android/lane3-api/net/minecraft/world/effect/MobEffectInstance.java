// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.effect;

import net.minecraft.core.Holder;

/** An effect given for a time at a strength. */
public class MobEffectInstance {
    private final Holder<MobEffect> effect;
    private final int duration;
    private final int amplifier;
    private final boolean visible;

    public MobEffectInstance(Holder<MobEffect> effect) {
        this(effect, 0, 0, false, true);
    }

    public MobEffectInstance(Holder<MobEffect> effect, int duration) {
        this(effect, duration, 0, false, true);
    }

    public MobEffectInstance(Holder<MobEffect> effect, int duration, int amplifier) {
        this(effect, duration, amplifier, false, true);
    }

    public MobEffectInstance(Holder<MobEffect> effect, int duration, int amplifier, boolean ambient, boolean visible) {
        this(effect, duration, amplifier, ambient, visible, visible);
    }

    public MobEffectInstance(Holder<MobEffect> effect, int duration, int amplifier, boolean ambient, boolean visible, boolean icon) {
        this.effect = effect;
        this.duration = duration;
        this.amplifier = amplifier;
        this.visible = visible;
    }

    public Holder<MobEffect> getEffect() {
        return effect;
    }

    public int getDuration() {
        return duration;
    }

    public int getAmplifier() {
        return amplifier;
    }

    public boolean isVisible() {
        return visible;
    }
}
