import useSWR, { SWRResponse } from "swr";

const apiEndpoint = "/api";

export type File = {
  directory: string;
  filename: string;
  type: "file" | "directory";
};

export type ListResult = File[];

export function useListApi(
  pathFragments: string[]
): SWRResponse<ListResult, any> {
  return useSWR<ListResult>([pathFragments], (pathFragments) =>
    fetch(
      `${apiEndpoint}/list?path=${encodeURIComponent(pathFragments.join("/"))}`
    ).then((res) => res.json())
  );
}

export type SearchResult = File[];

export function useSearchApi(query: string): SWRResponse<SearchResult, any> {
  return useSWR<SearchResult>(query, (query) =>
    fetch(`${apiEndpoint}/search?query=${encodeURIComponent(query)}`).then(
      (res) => res.json()
    )
  );
}
