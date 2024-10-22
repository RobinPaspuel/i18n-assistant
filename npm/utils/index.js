export const fetchLatestVersion = async () => {
  const response = await fetch('https://api.github.com/repos/RobinPaspuel/i18n-assistant/releases/latest');
  if (!response.ok) {
    throw new Error(`Failed to fetch latest release: ${response.statusText}`);
  }
  const data = await response.json();
  return data.tag_name.startsWith('v') ? data.tag_name.substring(1) : data.tag_name;
};
