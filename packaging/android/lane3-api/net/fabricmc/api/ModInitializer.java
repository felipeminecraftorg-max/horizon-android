// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.fabricmc.api;

/** A Fabric mod's common entrypoint ({@code main} in fabric.mod.json). */
@FunctionalInterface
public interface ModInitializer {
    void onInitialize();
}
