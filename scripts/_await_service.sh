RED=$(tput setaf 1)
GREEN=$(tput setaf 2)
RESET=$(tput sgr0)

# Wait for a service to be up by polling docker logs for presence of a search string
await_service() {
  local container_name="$1"
  local log_search="$2"
  local count=0

  echo -n "Waiting for $container_name..."
  while ! docker compose logs "$container_name" | grep -F "$log_search" > /dev/null; do
    echo -n "."
    sleep 2
    ((++count))

    if [[ "$count" -gt 20 ]]; then
      echo " [ ${RED}FAILED${RESET} ]"
      docker compose logs "$container_name" >&2
      return 1
    fi
  done

  echo " [ ${GREEN}OK${RESET} ]"
  return 0
}
