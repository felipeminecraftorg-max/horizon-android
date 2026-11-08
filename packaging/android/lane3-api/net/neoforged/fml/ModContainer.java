// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml;

import net.neoforged.bus.api.IEventBus;

/** A mod as the loader holds it. */
public class ModContainer {
    private final String id;
    private final IEventBus bus;

    public ModContainer(String id, IEventBus bus) {
        this.id = id;
        this.bus = bus;
    }

    public String getModId() {
        return id;
    }

    public String getNamespace() {
        return id;
    }

    public IEventBus getEventBus() {
        return bus;
    }
}
