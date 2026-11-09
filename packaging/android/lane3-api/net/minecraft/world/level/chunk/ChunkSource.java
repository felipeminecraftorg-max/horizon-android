// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.chunk;

/** A level's chunks: the server's have every chunk a player reaches. */
public class ChunkSource {
    private final boolean loaded;

    public ChunkSource(boolean loaded) {
        this.loaded = loaded;
    }

    public boolean hasChunk(int x, int z) {
        return loaded;
    }
}
