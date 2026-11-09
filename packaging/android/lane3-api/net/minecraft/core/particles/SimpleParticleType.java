// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core.particles;

/** A particle with nothing more than its type. */
public class SimpleParticleType extends ParticleType<SimpleParticleType> implements ParticleOptions {
    public SimpleParticleType(String id) {
        super(id);
    }

    /** A mod's: named when it is registered. */
    public SimpleParticleType(boolean overrideLimiter) {
        super(overrideLimiter);
    }

    @Override
    public SimpleParticleType getType() {
        return this;
    }

    /** A constant of the game's ({@code ParticleTypes.SMOKE}): the particle of the id its field names. */
    public static SimpleParticleType horizonConstant(String owner, String field) {
        return owner.endsWith("/ParticleTypes") ? new SimpleParticleType("minecraft:" + field.toLowerCase(java.util.Locale.ROOT)) : null;
    }
}
