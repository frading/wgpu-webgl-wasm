/**
 * Log levels supported by the logger
 */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

/**
 * Logger configuration
 */
export interface LoggerConfig {
  /** DOM element to append log entries to (optional) */
  container?: HTMLElement | null;
  /** Minimum log level to display */
  minLevel?: LogLevel;
}

const levelPriority: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

let logContainer: HTMLElement | null = null;
let minLogLevel: LogLevel = 'debug';

/**
 * Configure the logger
 */
export function configureLogger(config: LoggerConfig): void {
  if (config.container !== undefined) {
    logContainer = config.container;
  }
  if (config.minLevel !== undefined) {
    minLogLevel = config.minLevel;
  }
}

/**
 * Log a message at the specified level
 */
export function log(level: LogLevel, ...args: unknown[]): void {
  // Check if level is high enough
  if (levelPriority[level] < levelPriority[minLogLevel]) {
    return;
  }

  // Log to console
  const message = args.map((arg) => (typeof arg === 'string' ? arg : JSON.stringify(arg))).join(' ');
  switch (level) {
    case 'debug':
      console.debug(...args);
      break;
    case 'info':
      console.info(...args);
      break;
    case 'warn':
      console.warn(...args);
      break;
    case 'error':
      console.error(...args);
      break;
  }

  // Log to DOM container if configured
  if (logContainer) {
    const entry = document.createElement('div');
    entry.className = `log-entry log-${level}`;
    entry.textContent = `[${level.toUpperCase()}] ${message}`;
    logContainer.appendChild(entry);
    logContainer.scrollTop = logContainer.scrollHeight;
  }
}

/**
 * Convenience logging functions
 */
export const debug = (...args: unknown[]): void => log('debug', ...args);
export const info = (...args: unknown[]): void => log('info', ...args);
export const warn = (...args: unknown[]): void => log('warn', ...args);
export const error = (...args: unknown[]): void => log('error', ...args);
