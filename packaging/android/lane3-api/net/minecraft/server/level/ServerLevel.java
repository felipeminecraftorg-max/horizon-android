// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server.level;

import net.minecraft.world.level.Level;

public class ServerLevel extends Level implements net.minecraft.world.level.WorldGenLevel {
    @Override
    public ServerLevel getLevel() {
        return this;
    }

    public static final ServerLevel CURRENT = new ServerLevel();

    /** Particles the players near see (Horizon's {@code /particle}): how many it sent. */
    public <T extends net.minecraft.core.particles.ParticleOptions> int sendParticles(T particle, double x, double y, double z, int count,
            double dx, double dy, double dz, double speed) {
        if (particle == null || particle.getType() == null || particle.getType().horizonId() == null) return 0;
        boolean sent = dev.horizon.android.lane3.Native.command(String.format(java.util.Locale.ROOT,
                "particle %s %.3f %.3f %.3f %.3f %.3f %.3f %.3f %d normal", particle.getType().horizonId(), x, y, z, dx, dy, dz, speed,
                Math.max(0, count)));
        return sent ? 1 : 0;
    }

    public net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager getStructureManager() {
        return net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager.CURRENT;
    }

    public net.minecraft.server.MinecraftServer getServer() {
        return net.minecraft.server.MinecraftServer.CURRENT;
    }
}
