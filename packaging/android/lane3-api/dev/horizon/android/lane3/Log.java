// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package dev.horizon.android.lane3;

/** A mod's logger (SLF4J's and Log4j's), into Android's log through Horizon. */
public final class Log implements org.slf4j.Logger, org.apache.logging.log4j.Logger {
    private final String name;

    public Log(String name) {
        this.name = name;
    }

    private static String format(String format, Object... args) {
        if (format == null) return "null";
        StringBuilder out = new StringBuilder();
        int arg = 0;
        int i = 0;
        while (i < format.length()) {
            if (format.startsWith("{}", i) && arg < args.length) {
                out.append(args[arg++]);
                i += 2;
            } else {
                out.append(format.charAt(i++));
            }
        }
        if (arg < args.length && args[args.length - 1] instanceof Throwable t) {
            out.append(": ").append(t);
        }
        return out.toString();
    }

    private void line(String level, String text) {
        try {
            Native.log("[" + name + "] " + level + " " + text);
        } catch (UnsatisfiedLinkError e) {
            // Outside the app (a desktop check): nowhere to write.
        }
    }

    public String getName() { return name; }
    public boolean isTraceEnabled() { return false; }
    public boolean isDebugEnabled() { return false; }
    public boolean isInfoEnabled() { return true; }
    public boolean isWarnEnabled() { return true; }
    public boolean isErrorEnabled() { return true; }
    public void trace(String msg) {}
    public void trace(String format, Object arg) {}
    public void trace(String format, Object arg1, Object arg2) {}
    public void trace(String format, Object... arguments) {}
    public void trace(String msg, Throwable t) {}
    public void debug(String msg) {}
    public void debug(String format, Object arg) {}
    public void debug(String format, Object arg1, Object arg2) {}
    public void debug(String format, Object... arguments) {}
    public void debug(String msg, Throwable t) {}
    public void info(String msg) { line("INFO", msg); }
    public void info(String format, Object arg) { line("INFO", format(format, arg)); }
    public void info(String format, Object arg1, Object arg2) { line("INFO", format(format, arg1, arg2)); }
    public void info(String format, Object... arguments) { line("INFO", format(format, arguments)); }
    public void info(String msg, Throwable t) { line("INFO", msg + ": " + t); }
    public void warn(String msg) { line("WARN", msg); }
    public void warn(String format, Object arg) { line("WARN", format(format, arg)); }
    public void warn(String format, Object arg1, Object arg2) { line("WARN", format(format, arg1, arg2)); }
    public void warn(String format, Object... arguments) { line("WARN", format(format, arguments)); }
    public void warn(String msg, Throwable t) { line("WARN", msg + ": " + t); }
    public void error(String msg) { line("ERROR", msg); }
    public void error(String format, Object arg) { line("ERROR", format(format, arg)); }
    public void error(String format, Object arg1, Object arg2) { line("ERROR", format(format, arg1, arg2)); }
    public void error(String format, Object... arguments) { line("ERROR", format(format, arguments)); }
    public void error(String msg, Throwable t) { line("ERROR", msg + ": " + t); }
    public void fatal(String msg) { line("FATAL", msg); }
}
