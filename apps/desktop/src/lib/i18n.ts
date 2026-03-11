export type Locale = "en" | "ru";

const translations = {
  en: {
    appName: "Codex Switcher",
    profiles: "Profiles",
    quickSwitch: "Quick Switch",
    reload: "Reload",
    settings: "Settings",
    connected: "Connected",
    error: "Error",
    refreshed: "Refreshed",
    partialRefresh: "Partial refresh",
    profileDataUpdated: "Profile data updated.",
    someDataCouldNotBeLoaded: "Some data could not be loaded.",
    refreshedProfiles: "Refreshed profiles",
    previewSwitch: "Preview switch",
    loading: "Loading...",
    switching: "Switching...",
    switchTo: "Switch",
    switched: "Switched",
    switchIncomplete: "Switch incomplete",
    switchFailed: "Switch failed",
    previewFailed: "Preview failed",
    reloadFailed: "Reload failed",
    retry: "Retry",
    unableToConnect: "Unable to connect",
    bridgeNotResponding: "Bridge is not responding.",
    connectingToBridge: "Connecting to bridge...",
    selectProfile: "Select a profile",
    selectProfileHint: "Select a profile from the sidebar to view its details.",
    noProfilesFound: "No profiles found",
    createProfileViaCli: "Create a profile via the CLI.",
    recent: "Recent",
    noRecentActions: "No recent actions yet.",
    recentEvents: "Recent Events",
    healthyUsage: "Healthy usage",
    moderateUsage: "Moderate usage",
    lowHeadroom: "Low headroom - consider switching",
    healthyWindow: "Healthy window",
    windowNarrowing: "Window narrowing",
    windowNearlyFull: "Window nearly full - switch soon",
    sevenDayHeadroom: "7-day headroom",
    fiveHourHeadroom: "5-hour headroom",
    current: "Current",
    available: "Available",
    reserved: "Reserved",
    reservedForAnotherSession: "Reserved for another session",
    quickSwitchTitle: "Quick Switch",
    quickSwitchDesc:
      "Switch directly to the best non-reserved profile using the shared Rust runtime.",
    quickSwitchTip:
      "Quick Switch uses the same best-profile logic as the CLI switch command.",
    bestSwitchCandidate: "Best switch candidate",
    switchBestProfile: "Switch to best profile",
    noAvailableProfiles: "No available profiles to switch to.",
    reloadSessions: "Reload Sessions",
    reloadDesc: "Restart IDE sessions to pick up profile changes.",
    lastReloaded: "Last reloaded",
    reloadTip:
      "Tip: Reload after switching profiles to apply changes in your IDE sessions.",
    noReloadTargets: "No reload targets available.",
    reserve: "Reserve",
    unreserve: "Unreserve",
    reserveSuccess: "Reserved",
    unreserveSuccess: "Unreserved",
    profileReserved: "Profile marked as reserved.",
    profileUnreserved: "Profile is now available for switching.",
    settingsTitle: "Settings",
    settingsDesc: "Configure Codex Switcher behavior.",
    language: "Language",
    reloadAfterSwitch: "Reload after switch",
    reloadAfterSwitchDesc:
      "Automatically reload IDE sessions after switching profiles.",
    primaryReloadTarget: "Primary reload target",
    primaryReloadTargetDesc: "Which IDE session to reload by default.",
    sortBy: "Sort by",
    sortByRating: "Rating (best first)",
    sortByName: "Name (A-Z)",
    sortByUsage: "Usage (most remaining)",
    theme: "Theme",
    dark: "Dark",
    rankLabel: "Rank",
    score: "Score",
    on: "On",
    off: "Off",
    all: "All",
    workspace: "Workspace",
    refreshTooltip: "Refresh all data (Ctrl+R)",
    recommended: "recommended",
    viewNavigation: "View navigation",
    statusBar: "Status bar",
    resizeSidebar: "Resize profile sidebar",
    activeBadge: "active",
    reserveLocally: "Reserve locally",
    clearLocalReserve: "Clear local reserve",
    localReserveNote: "Local MVP only. Shared reserve state is not wired yet.",
    closePreview: "Close preview",
    dismissToast: "Dismiss notification",
  },
  ru: {
    appName: "Codex Switcher",
    profiles: "Профили",
    quickSwitch: "Быстрое переключение",
    reload: "Перезагрузка",
    settings: "Настройки",
    connected: "Подключено",
    error: "Ошибка",
    refreshed: "Обновлено",
    partialRefresh: "Частичное обновление",
    profileDataUpdated: "Данные профилей обновлены.",
    someDataCouldNotBeLoaded: "Часть данных не удалось загрузить.",
    refreshedProfiles: "Профили обновлены",
    previewSwitch: "Предпросмотр переключения",
    loading: "Загрузка...",
    switching: "Переключение...",
    switchTo: "Переключить",
    switched: "Переключено",
    switchIncomplete: "Переключение завершено частично",
    switchFailed: "Не удалось переключить профиль",
    previewFailed: "Не удалось получить предпросмотр",
    reloadFailed: "Не удалось выполнить перезагрузку",
    retry: "Повторить",
    unableToConnect: "Не удалось подключиться",
    bridgeNotResponding: "Нативный мост не отвечает.",
    connectingToBridge: "Подключение к нативному мосту...",
    selectProfile: "Выберите профиль",
    selectProfileHint: "Выберите профиль в боковой панели, чтобы открыть его подробности.",
    noProfilesFound: "Профили не найдены",
    createProfileViaCli: "Создайте профиль через CLI.",
    recent: "Недавние",
    noRecentActions: "Пока нет недавних действий.",
    recentEvents: "Недавние события",
    healthyUsage: "Лимиты в хорошем состоянии",
    moderateUsage: "Лимиты заметно расходуются",
    lowHeadroom: "Запас лимита низкий, стоит переключиться",
    healthyWindow: "Окно лимита в норме",
    windowNarrowing: "Окно лимита сужается",
    windowNearlyFull: "Окно лимита почти исчерпано, переключитесь скоро",
    sevenDayHeadroom: "Запас на 7 дней",
    fiveHourHeadroom: "Запас на 5 часов",
    current: "Текущий",
    available: "Доступен",
    reserved: "Резерв",
    reservedForAnotherSession: "Зарезервирован для другой сессии",
    quickSwitchTitle: "Быстрое переключение",
    quickSwitchDesc:
      "Сразу переключайтесь на лучший незарезервированный профиль через общий Rust runtime.",
    quickSwitchTip:
      "Быстрое переключение использует ту же логику выбора лучшего профиля, что и команда CLI switch.",
    bestSwitchCandidate: "Лучший кандидат для переключения",
    switchBestProfile: "Переключиться на лучший профиль",
    noAvailableProfiles: "Нет доступных профилей для переключения.",
    reloadSessions: "Перезагрузка сессий",
    reloadDesc: "Перезапустите IDE-сессии, чтобы они подхватили изменения профиля.",
    lastReloaded: "Последняя перезагрузка",
    reloadTip:
      "Совет: после смены профиля перезагрузите IDE, чтобы она применила новые учетные данные.",
    noReloadTargets: "Нет доступных целей для перезагрузки.",
    reserve: "Резерв",
    unreserve: "Снять резерв",
    reserveSuccess: "Профиль зарезервирован",
    unreserveSuccess: "Резерв снят",
    profileReserved: "Профиль исключен из автопереключения.",
    profileUnreserved: "Профиль снова доступен для переключения.",
    settingsTitle: "Настройки",
    settingsDesc: "Настройте поведение Codex Switcher.",
    language: "Язык",
    reloadAfterSwitch: "Перезагружать после переключения",
    reloadAfterSwitchDesc:
      "Автоматически перезагружать IDE-сессии после смены профиля.",
    primaryReloadTarget: "Основная цель перезагрузки",
    primaryReloadTargetDesc: "Какую IDE-сессию перезагружать по умолчанию.",
    sortBy: "Сортировка",
    sortByRating: "Рейтинг (лучшие сверху)",
    sortByName: "Имя (А-Я)",
    sortByUsage: "Остаток лимитов",
    theme: "Тема",
    dark: "Темная",
    rankLabel: "Ранг",
    score: "Оценка",
    on: "Вкл",
    off: "Выкл",
    all: "Все",
    workspace: "Рабочее пространство",
    refreshTooltip: "Обновить все данные (Ctrl+R)",
    recommended: "рекомендуется",
    viewNavigation: "Навигация по разделам",
    statusBar: "Строка состояния",
    resizeSidebar: "Изменить ширину панели профилей",
    activeBadge: "активен",
    reserveLocally: "Зарезервировать локально",
    clearLocalReserve: "Снять локальный резерв",
    localReserveNote:
      "Пока это только локальный MVP. Общий резерв еще не подключен.",
    closePreview: "Закрыть предпросмотр",
    dismissToast: "Закрыть уведомление",
  },
} as const;

type TranslationMap = typeof translations;
type EnglishKeys = keyof TranslationMap["en"];
type RussianKeys = keyof TranslationMap["ru"];

function validateTranslationParity() {
  const enKeys = Object.keys(translations.en) as EnglishKeys[];
  const ruKeys = new Set(Object.keys(translations.ru) as RussianKeys[]);
  const missingInRu = enKeys.filter((key) => !ruKeys.has(key));

  if (missingInRu.length > 0) {
    console.error(`[i18n] Russian locale is missing keys: ${missingInRu.join(", ")}`);
  }
}

validateTranslationParity();

export type TranslationKey = keyof typeof translations.en;

export function t(locale: Locale, key: TranslationKey): string {
  return translations[locale][key] ?? translations.en[key] ?? key;
}

export function getLocaleLabel(locale: Locale): string {
  return locale === "ru" ? "Русский" : "English";
}

function russianProfileWord(count: number): string {
  const mod10 = count % 10;
  const mod100 = count % 100;

  if (mod10 === 1 && mod100 !== 11) {
    return "профиль";
  }

  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
    return "профиля";
  }

  return "профилей";
}

export function formatWorkspaceLabel(locale: Locale, profileCount: number): string {
  if (locale === "en") {
    return `Shared runtime: ${profileCount} profile${profileCount === 1 ? "" : "s"}`;
  }

  return `Общий рантайм: ${profileCount} ${russianProfileWord(profileCount)}`;
}

export function getReloadTargetLabel(
  locale: Locale,
  targetId: string,
  fallbackLabel: string,
): string {
  if (locale === "en") {
    return fallbackLabel;
  }

  switch (targetId) {
    case "codex":
      return "Перезагрузить Codex";
    case "cursor":
      return "Перезагрузить Cursor";
    case "all":
      return "Перезагрузить все";
    default:
      return fallbackLabel;
  }
}

export function getReloadTargetDescription(
  locale: Locale,
  targetId: string,
  fallbackDescription: string,
): string {
  if (locale === "en") {
    return fallbackDescription;
  }

  switch (targetId) {
    case "codex":
      return "Обновить сессию Codex после переключения аккаунта.";
    case "cursor":
      return "Обновить Cursor после изменения авторизации редактора.";
    case "all":
      return "Обновить все доступные IDE-сессии после смены профиля.";
    default:
      return localizeRuntimeText(locale, fallbackDescription);
  }
}

export function getAvailabilityLabel(
  locale: Locale,
  tag: string,
  fallbackLabel: string,
): string {
  if (locale === "en") {
    return fallbackLabel;
  }

  switch (tag) {
    case "tokenUnreadable":
      return "Ошибка токена";
    case "apiKeyUnsupported":
      return "Только API key";
    case "freePlanUnsupported":
      return "Free-план";
    case "missingAccessToken":
      return "Нет auth token";
    case "missingAccountId":
      return "Нет account id";
    case "usageFetchError":
      return "Ошибка usage";
    case "missingFiveHourWindow":
      return "Нет окна 5ч";
    case "missingSevenDayWindow":
      return "Нет окна 7д";
    default:
      return fallbackLabel;
  }
}

const exactRuRuntimeTexts: Record<string, string> = {
  "Shared Rust reload services are ready.": "Сервисы перезагрузки готовы.",
  "Current auth session is not saved yet; the shared service still exposes it as the active profile.":
    "Текущая сессия авторизации еще не сохранена, но общий сервис уже показывает ее как активный профиль.",
  "No active profile could be derived from the current auth state.":
    "Не удалось определить активный профиль из текущего состояния авторизации.",
  "No eligible profile is available for automatic switching.":
    "Нет подходящего профиля для автоматического переключения.",
  "The current auth session is unsaved; save it first if you want label-based switching to stay stable.":
    "Текущая сессия авторизации не сохранена. Сначала сохраните ее, если хотите стабильное переключение по имени профиля.",
  "Reserved profiles stay out of automatic-switch candidacy until they are unreserved.":
    "Зарезервированные профили не участвуют в автопереключении, пока с них не снят резерв.",
  "Retry the shared refresh path before forcing a re-login.":
    "Сначала попробуйте общий путь обновления, прежде чем делать повторный вход.",
  "Usage-based switching does not support API key logins.":
    "Переключение по лимитам не поддерживает вход через API key.",
  "Usage-based switching requires a supported paid Cursor plan.":
    "Для переключения по лимитам нужен поддерживаемый платный план Cursor.",
  "Refresh usage windows or inspect account usage data before switching.":
    "Обновите окна лимитов или проверьте данные аккаунта перед переключением.",
  "Repair or resave the saved profile before attempting an automatic switch.":
    "Исправьте или заново сохраните профиль перед автоматическим переключением.",
  "At least one non-reserved profile with readable usage data is required for automatic switching.":
    "Для автоматического переключения нужен хотя бы один незарезервированный профиль с читаемыми данными по лимитам.",
  "Choose a profile before requesting a switch preview.":
    "Сначала выберите профиль, прежде чем запрашивать предпросмотр переключения.",
  "Choose a profile before executing a switch.":
    "Сначала выберите профиль, прежде чем выполнять переключение.",
  "The requested reload target is not part of the approved desktop bootstrap surface.":
    "Запрошенная цель перезагрузки не входит в утвержденную desktop bootstrap surface.",
  "Missing access token": "Отсутствует access token",
  "Missing account id": "Отсутствует account id",
  "Usage unavailable for API key login": "Usage недоступен для логина через API key",
  "Usage unavailable for free plan": "Usage недоступен для free-плана",
  "Missing 5h usage window": "Отсутствует usage-окно 5ч",
  "Missing 7d usage window": "Отсутствует usage-окно 7д",
  "Profile tokens are unreadable": "Токены профиля не читаются",
};

export function localizeRuntimeText(locale: Locale, text: string): string {
  if (locale === "en" || !text) {
    return text;
  }

  if (text in exactRuRuntimeTexts) {
    return exactRuRuntimeTexts[text];
  }

  const sharedRuntimeMatch = text.match(/^Shared runtime: (\d+) profile(?:s)?$/);
  if (sharedRuntimeMatch) {
    return formatWorkspaceLabel(locale, Number(sharedRuntimeMatch[1]));
  }

  const returnedProfilesMatch = text.match(
    /^Canonical Rust runtime returned (\d+) profile(?:s)? for the desktop shell\.$/,
  );
  if (returnedProfilesMatch) {
    const count = Number(returnedProfilesMatch[1]);
    return `Основной рантайм вернул ${count} ${russianProfileWord(count)} для приложения.`;
  }

  const activeProfileMatch = text.match(/^Active profile: (.+)\.$/);
  if (activeProfileMatch) {
    return `Активный профиль: ${activeProfileMatch[1]}.`;
  }

  const reservedProfilesMatch = text.match(
    /^(\d+) reserved profile(?:s)? stay out of auto-switch candidacy\.$/,
  );
  if (reservedProfilesMatch) {
    const count = Number(reservedProfilesMatch[1]);
    return `${count} ${
      count === 1 ? "зарезервированный профиль исключен" : "зарезервированных профиля исключены"
    } из автопереключения.`;
  }

  const activeSummaryMatch = text.match(
    /^(.+) is the current (active|available|reserved) profile exposed by the shared Rust query service(?: \((.+)\))?\.$/,
  );
  if (activeSummaryMatch) {
    const [, label, state, details] = activeSummaryMatch;
    const stateLabel =
      state === "active"
        ? "активный"
        : state === "reserved"
          ? "зарезервированный"
          : "доступный";

    if (details) {
      return `${label} — текущий ${stateLabel} профиль, который вернул общий сервис запросов (${localizeRuntimeText(locale, details)}).`;
    }

    return `${label} — текущий ${stateLabel} профиль, который вернул общий сервис запросов.`;
  }

  const activeAuthJsonMatch = text.match(
    /^(.+) is active in auth\.json but has not been saved as a reusable profile yet\.$/,
  );
  if (activeAuthJsonMatch) {
    return `${activeAuthJsonMatch[1]} активен в auth.json, но еще не сохранен как переиспользуемый профиль.`;
  }

  const alreadyActiveMatch = text.match(
    /^(.+) is already the active profile in the shared Rust runtime\.$/,
  );
  if (alreadyActiveMatch) {
    return `${alreadyActiveMatch[1]} уже является активным профилем в общем Rust runtime.`;
  }

  const reservedSwitchMatch = text.match(
    /^(.+) is reserved and excluded from automatic switching\.$/,
  );
  if (reservedSwitchMatch) {
    return `${reservedSwitchMatch[1]} зарезервирован и исключен из автоматического переключения.`;
  }

  const notSwitchableMatch = text.match(
    /^(.+) is not currently switchable \[([A-Z0-9_]+)\]: (.+)\.$/,
  );
  if (notSwitchableMatch) {
    return `${notSwitchableMatch[1]} сейчас недоступен для переключения [${notSwitchableMatch[2]}]: ${localizeRuntimeText(locale, notSwitchableMatch[3])}.`;
  }

  const bestCandidateMatch = text.match(
    /^(.+) is the current best switch candidate from the shared Rust runtime\.$/,
  );
  if (bestCandidateMatch) {
    return `${bestCandidateMatch[1]} сейчас является лучшим кандидатом для переключения по общему рантайму.`;
  }

  const betterCandidateMatch = text.match(
    /^(.+) is available, but (.+) is currently the best switch candidate\.$/,
  );
  if (betterCandidateMatch) {
    return `${betterCandidateMatch[1]} доступен, но ${betterCandidateMatch[2]} сейчас является лучшим кандидатом для переключения.`;
  }

  const noRecommendedCandidateMatch = text.match(
    /^(.+) is available, but no recommended auto-switch candidate could be derived\.$/,
  );
  if (noRecommendedCandidateMatch) {
    return `${noRecommendedCandidateMatch[1]} доступен, но рекомендуемого кандидата для автопереключения определить не удалось.`;
  }

  const notSwitchableFromRuntimeMatch = text.match(
    /^Profile '(.+)' is not currently switchable from the shared Rust runtime\.$/,
  );
  if (notSwitchableFromRuntimeMatch) {
    return `Профиль '${notSwitchableFromRuntimeMatch[1]}' сейчас недоступен для переключения из общего рантайма.`;
  }

  const loadedViaSwitchServiceMatch = text.match(
    /^Loaded (.+) via the shared Rust switch service and processed (.+) reload guidance\.$/,
  );
  if (loadedViaSwitchServiceMatch) {
    return `${loadedViaSwitchServiceMatch[1]} загружен через общий сервис переключения, инструкции по перезагрузке обработаны (${loadedViaSwitchServiceMatch[2]}).`;
  }

  const loadedViaSwitchServiceSimpleMatch = text.match(
    /^Loaded (.+) via the shared Rust switch service\.$/,
  );
  if (loadedViaSwitchServiceSimpleMatch) {
    return `${loadedViaSwitchServiceSimpleMatch[1]} загружен через общий сервис переключения.`;
  }

  const failedFetchUsageMatch = text.match(/^failed to fetch usage: http status: (\d+)$/i);
  if (failedFetchUsageMatch) {
    return `не удалось получить данные по лимитам: HTTP статус ${failedFetchUsageMatch[1]}`;
  }

  const nativeBridgeUnavailableMatch = text.match(
    /^The native desktop bridge is unavailable, so (.+)\.$/,
  );
  if (nativeBridgeUnavailableMatch) {
    return `Нативный мост рабочего стола недоступен, поэтому ${nativeBridgeUnavailableMatch[1]}.`;
  }

  return text;
}
