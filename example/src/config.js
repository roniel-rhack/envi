// Example source file for the scanner to detect env var usage
const config = {
  database: {
    url: process.env.DATABASE_URL,
    poolSize: parseInt(process.env.DATABASE_POOL_SIZE, 10),
  },
  api: {
    key: process.env.API_KEY,
    stripeKey: process.env.STRIPE_SECRET_KEY,
  },
  app: {
    port: process.env.PORT || 3000,
    host: process.env.HOST || '0.0.0.0',
    env: process.env.NODE_ENV || 'development',
  },
  redis: {
    url: process.env.REDIS_URL,
  },
  auth: {
    jwtSecret: process.env.JWT_SECRET,
    sessionTimeout: process.env.SESSION_TIMEOUT,
  },
  email: {
    host: process.env.SMTP_HOST,
    port: process.env.SMTP_PORT,
    user: process.env.SMTP_USER,
    password: process.env.SMTP_PASSWORD,
  },
  features: {
    notifications: process.env.ENABLE_NOTIFICATIONS === 'true',
    analytics: process.env.ENABLE_ANALYTICS === 'true',
  },
  // This var is used in code but NOT defined in .env
  newFeature: process.env.NEW_FEATURE_FLAG,
  cdnUrl: process.env.CDN_URL,
};

module.exports = config;
