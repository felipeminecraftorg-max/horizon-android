// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.fabricmc.loader.api;

import java.nio.file.Path;
import net.fabricmc.api.EnvType;

/** The loader as Horizon's lane 3 has it: the mods it loaded, and their folders. */
public interface FabricLoader {
    static FabricLoader getInstance() {
        return dev.horizon.android.lane3.Lane3.LOADER;
    }

    boolean isModLoaded(String id);

    EnvType getEnvironmentType();

    Path getConfigDir();

    Path getGameDir();

    boolean isDevelopmentEnvironment();
}
