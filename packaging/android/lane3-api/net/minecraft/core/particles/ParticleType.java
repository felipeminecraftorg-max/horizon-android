// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core.particles;

/** A kind of particle, by its id. */
public abstract class ParticleType<T extends ParticleOptions> {
    private final String id;

    protected ParticleType(String id) {
        this.id = id;
    }

    /** Its id ({@code minecraft:smoke}), as `/particle` takes it. */
    public String horizonId() {
        return id;
    }
}
