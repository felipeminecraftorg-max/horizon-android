// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core.particles;

/** A kind of particle, by its id. */
public abstract class ParticleType<T extends ParticleOptions> {
    private String id;

    protected ParticleType(String id) {
        this.id = id;
    }

    /** A mod's particle type: its id is the one it is registered under (lane 3 sets it). */
    protected ParticleType(boolean overrideLimiter) {}

    protected ParticleType() {}

    /** Its id ({@code minecraft:smoke}), as `/particle` takes it. */
    public String horizonId() {
        return id;
    }

    /** The id lane 3 registered this type under. */
    public final void horizonId(String id) {
        if (this.id == null) this.id = id;
    }
}
