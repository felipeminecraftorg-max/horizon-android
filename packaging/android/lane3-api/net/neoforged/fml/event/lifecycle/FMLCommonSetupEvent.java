// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml.event.lifecycle;

/** The mods' common setup, after they registered: work queued for it runs at once. */
public class FMLCommonSetupEvent extends net.neoforged.bus.api.Event {
    public java.util.concurrent.CompletableFuture<Void> enqueueWork(Runnable work) {
        work.run();
        return java.util.concurrent.CompletableFuture.completedFuture(null);
    }
}
