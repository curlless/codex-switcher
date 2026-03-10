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
    previewFailed: "Не удалось построить предпросмотр",
    reloadFailed: "Не удалось выполнить перезагрузку",
    retry: "Повторить",
    unableToConnect: "Не удалось подключиться",
    bridgeNotResponding: "Нативный bridge не отвечает.",
    connectingToBridge: "Подключение к bridge...",
    selectProfile: "Выберите профиль",
    selectProfileHint:
      "Выберите профиль в боковой панели, чтобы открыть его подробности.",
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
    reloadDesc:
      "Перезапустите IDE-сессии, чтобы они подхватили изменения профиля.",
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
    primaryReloadTargetDesc:
      "Какую IDE-сессию перезагружать по умолчанию.",
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
    viewNavigation: "Навигация по видам",
    statusBar: "Строка состояния",
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
    console.error(
      `[i18n] Russian locale is missing keys: ${missingInRu.join(", ")}`,
    );
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
