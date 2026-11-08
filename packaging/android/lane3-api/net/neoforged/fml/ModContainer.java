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

    /** A settings file: lane 3 answers its defaults. */
    public void registerConfig(net.neoforged.fml.config.ModConfig.Type type, net.neoforged.fml.config.IConfigSpec spec) {}

    public void registerConfig(net.neoforged.fml.config.ModConfig.Type type, net.neoforged.fml.config.IConfigSpec spec, String file) {}

    public <T> void registerExtensionPoint(Class<? extends T> point, T extension) {}

    public <T> void registerExtensionPoint(Class<? extends T> point, java.util.function.Supplier<? extends T> extension) {}
}
