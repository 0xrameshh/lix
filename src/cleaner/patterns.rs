pub static API_KEY_PATTERNS: &[(&str, &str)] = &[
    (
        r"(?P<prefix>sk-or-v1-)(?P<body>[A-Za-z0-9_-]{16,})",
        "api_key",
    ),
    (
        r"(?P<prefix>sk-ant-api03-)(?P<body>[A-Za-z0-9_-]{16,})",
        "api_key",
    ),
    (
        r"(?P<prefix>sk-proj-)(?P<body>[A-Za-z0-9_-]{16,})",
        "api_key",
    ),
    (r"(?P<prefix>sk-)(?P<body>[A-Za-z0-9_-]{20,})", "api_key"),
    (r"(?P<prefix>hf_)(?P<body>[A-Za-z0-9]{20,})", "api_key"),
    (r"(?P<prefix>gsk_)(?P<body>[A-Za-z0-9]{20,})", "api_key"),
    (
        r"(?P<prefix>github_pat_)(?P<body>[A-Za-z0-9_]{20,})",
        "api_key",
    ),
    (
        r"(?P<prefix>gh[pousr]_)(?P<body>[A-Za-z0-9_]{20,})",
        "api_key",
    ),
    (r"(?P<prefix>glpat-)(?P<body>[A-Za-z0-9_-]{20,})", "api_key"),
    (r"(?P<prefix>lin_api_)(?P<body>[A-Za-z0-9]{20,})", "api_key"),
    (r"(?P<prefix>npm_)(?P<body>[A-Za-z0-9]{20,})", "api_key"),
    (r"(?P<prefix>pypi-)(?P<body>[A-Za-z0-9_-]{20,})", "api_key"),
    (
        r"(?P<prefix>sk_(?:live|test)_)(?P<body>[A-Za-z0-9]{16,})",
        "api_key",
    ),
    (
        r"(?P<prefix>(?:rk|pk)_(?:live|test)_)(?P<body>[A-Za-z0-9]{16,})",
        "api_key",
    ),
    (r"(?P<prefix>whsec_)(?P<body>[A-Za-z0-9]{16,})", "api_key"),
    (r"(?P<prefix>re_)(?P<body>[A-Za-z0-9]{20,})", "api_key"),
    (
        r"(?P<prefix>sq0(?:atp|csp)-)(?P<body>[A-Za-z0-9_-]{20,})",
        "api_key",
    ),
    (
        r"(?P<prefix>xox[baprs]-)(?P<body>[A-Za-z0-9-]{20,})",
        "api_key",
    ),
    (r"(?P<prefix>AIza)(?P<body>[A-Za-z0-9_-]{20,})", "api_key"),
    (
        r"(?P<prefix>GOCSPX-)(?P<body>[A-Za-z0-9_-]{20,})",
        "api_key",
    ),
    (r"(?P<prefix>ctx7sk-)(?P<body>[A-Za-z0-9-]{20,})", "api_key"),
    (
        r"(?P<prefix>(?:AKIA|ASIA))(?P<body>[A-Z0-9]{16})(?:$|[^A-Za-z0-9])",
        "api_key",
    ),
    (
        r"(?P<prefix>SK)(?P<body>[0-9a-fA-F]{32})(?:$|[^A-Za-z0-9])",
        "api_key",
    ),
    (
        r"(?P<prefix>SG\.)(?P<body>[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{20,})",
        "api_key",
    ),
];

pub static SENSITIVE_KEYS: &[&str] = &[
    "api_key",
    "token",
    "secret",
    "password",
    "access_token",
    "access_key",
    "admin_password",
    "app_secret",
    "auth_secret",
    "auth_token",
    "aws_secret_access_key",
    "client_secret",
    "cookie_secret",
    "dashboard_password",
    "db_password",
    "django_secret_key",
    "jwt_secret",
    "mail_password",
    "mysql_password",
    "nextauth_secret",
    "secret_key",
    "session_secret",
    "stripe_secret",
    "stripe_api_key",
    "webhook_secret",
];
